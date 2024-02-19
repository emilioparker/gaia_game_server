
use std::collections::HashMap;
use std::sync::Arc;

use futures_util::lock::Mutex;
use hyper::http::Error;
use hyper::{Request, body, server::conn::AddrStream};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;

use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};

use crate::chat::chat_entry::{ChatEntry, CHAT_ENTRY_SIZE};
use crate::definitions::definitions_container::DefinitionsData;
use crate::map::GameMap;
use crate::map::map_entity::MapEntity;
use crate::map::tetrahedron_id::TetrahedronId;
use crate::character::character_entity::InventoryItem;
use crate::ServerState;
use crate::tower::tower_entity::{TowerEntity, TOWER_ENTITY_SIZE};

pub mod characters;
pub mod map;
pub mod towers;
pub mod chat;

pub const CHAT_STORAGE_SIZE: usize = 100;

#[derive(Deserialize, Serialize, Debug)]
struct ClientVersionRequest 
{
    client_version: u16,
}

#[derive(Deserialize, Serialize, Debug)]
struct ClientVersionResponse 
{
    server_version:u16,
}

#[derive(Deserialize, Serialize, Debug)]
struct SellItemRequest 
{
    player_token: String,
    character_id:u16,
    old_item_id:u32,
    amount:u16,
}

#[derive(Deserialize, Serialize, Debug)]
struct SellItemResponse 
{
    new_item_id:u32,
    amount:u16,
}

#[derive(Deserialize, Serialize, Debug)]
struct DefinitionRequest 
{
    name: String,
    version: u16,
}

struct ChatStorage 
{
    index:usize,
    count:usize,
    record:[u8;CHAT_ENTRY_SIZE * CHAT_STORAGE_SIZE]
}

#[derive(Clone)]
pub struct AppContext 
{
    cached_presentation_data: Arc<Mutex<Vec<u8>>>,// we will keep a copy and update it more or less frequently.
    working_game_map : Arc<GameMap>,
    storage_game_map : Arc<GameMap>,
    server_state : Arc<ServerState>,
    definitions_data : DefinitionsData,
    // tx_mc_webservice_realtime : Sender<MapCommand>,
    db_client : mongodb ::Client,
    temp_regions : Arc::<HashMap::<TetrahedronId, Arc<Mutex<TempMapBuffer>>>>,
    temp_towers : Arc::<Mutex::<(usize,[u8;100000])>>,
    old_messages : Arc::<Mutex::<HashMap<u8, ChatStorage>>>//index, offset, count, 20 messages
}

async fn handle_sell_item(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> 
{
    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: SellItemRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);

    let mut players = context.working_game_map.players.lock().await;

    if let Some(player) = players.get_mut(&data.character_id) {
        // println!("selling player {:?}", player);

        // let mut updated_player_entity = player.clone();
        let result = player.remove_inventory_item(InventoryItem
        {
            item_id: data.old_item_id,
            level: 1,
            quality: 1,
            amount: data.amount,
        });// add soft currency

        if !result 
        {
            let mut response = Response::new(Body::from(String::from("transaction failed")));
            *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            return Ok(response);
        }

        player.add_inventory_item(InventoryItem{
            item_id: 0,
            level: 1,
            quality: 1,
            amount: 1,
        });// add soft currency

        let saved_char = SellItemResponse{
            new_item_id: 0,
            amount: 1,
        };

        let response = serde_json::to_vec(&saved_char).unwrap();
        Ok(Response::new(Body::from(response)))
    }
    else {
        let mut response = Response::new(Body::from(String::from("Player not found")));
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
    }
}

async fn handle_check_version(_context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> 
{
    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: ClientVersionRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);

    let response = ClientVersionResponse
    {
        server_version : 5
    };
    let response = serde_json::to_vec(&response).unwrap();
    Ok(Response::new(Body::from(response)))
}


async fn handle_definition_request(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> 
{
    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let result : Result<DefinitionRequest, _> = serde_json::from_slice(&data);
    if let Ok(data) = result
    {
        println!("handling request {:?}", data);

        if let Some(definition_data)= context.definitions_data.definition_versions.get(&data.name)
        {
            if definition_data.version == data.version && data.name == "character_progression"
            {
                return Ok(Response::new(Body::from(context.definitions_data.character_progression_data)));
            }
            else if definition_data.version == data.version && data.name == "definition_versions"
            {
                return Ok(Response::new(Body::from(context.definitions_data.definition_versions_data)));
            }
            else if definition_data.version == data.version && data.name == "props"
            {
                return Ok(Response::new(Body::from(context.definitions_data.props_data)));
            }
            else if definition_data.version == data.version && data.name == "mob_progression"
            {
                return Ok(Response::new(Body::from(context.definitions_data.mob_progression_data)));
            }
            else if definition_data.version == data.version && data.name == "main_paths"
            {
                return Ok(Response::new(Body::from(context.definitions_data.main_paths_data)));
            }
            else
            {
                let mut response = Response::new(Body::from(String::from("incorrect_definition_version")));
                *response.status_mut() = StatusCode::NOT_FOUND;
                return Ok(response);
            }
        }
        else {
            let mut response = Response::new(Body::from(String::from("definition_details_not_found")));
            *response.status_mut() = StatusCode::NOT_FOUND;
            return Ok(response);
        }
    }
    else
    {
        let mut response = Response::new(Body::from(String::from("definition_not_found")));
        *response.status_mut() = StatusCode::NOT_ACCEPTABLE;
        return Ok(response);
    }

}

async fn route(context: AppContext, req: Request<Body>) -> Result<Response<Body>, Error> {

    let uri = req.uri().to_string();
    let mut data = uri.split("/");
    data.next();
    if let Some(route) = data.next() {
        let rest : Vec<&str> = data.collect();
        match route {
            "region" => map::handle_region_request(context, rest).await,
            "temp_regions" => map::handle_temp_region_request(context, rest).await,
            "definitions" => handle_definition_request(context, req).await,
            "character_data" => characters::handle_characters_request(context).await,
            "player_creation" => characters::handle_create_player(context, req).await,
            "player_data" => characters::handle_player_request(context, req).await,
            "character_creation" => characters::handle_create_character(context, req).await,
            "join_with_character" => characters::handle_login_character(context, req).await,
            "towers" => towers::handle_request_towers(context, req).await,
            "temp_towers" => towers::handle_temp_tower_request(context).await,
            "sell_item" => handle_sell_item(context, req).await,
            "chat_record" => chat::handle_chat_record_request(context, rest).await,
            "exchange_skill_points" => characters::exchange_skill_points(context, req).await,
            "check_version" => handle_check_version(context, req).await,
            _ => {
                let mut response = Response::new(Body::from(String::from("route not found")));
                *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
                return Ok(response);
            }
        }
    }
    else {
        let mut response = Response::new(Body::from(String::from("missing route")));
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
    }
}

pub struct TempMapBuffer { 
    pub index : usize,
    pub tile_to_index : HashMap<TetrahedronId, usize>,
    pub buffer : [u8;100000]
}

impl TempMapBuffer {
    pub fn new() -> TempMapBuffer
    {
        TempMapBuffer { index: 0, buffer: [0; 100000], tile_to_index : HashMap::new() }
    }
}

pub fn start_server(
    presentation_cache : Vec<u8>,
    working_map: Arc<GameMap>,
    storage_map: Arc<GameMap>,
    server_state: Arc<ServerState>,
    db_client : mongodb :: Client,
    definitions_data : DefinitionsData,
    mut rx_me_realtime_webservice : Receiver<MapEntity>,
    mut rx_te_realtime_webservice : Receiver<TowerEntity>,
    mut rx_ce_realtime_webservice : Receiver<ChatEntry>,
    // tx_mc_webservice_gameplay : Sender<MapCommand>,
    mut rx_saved_me_longterm_webservice : Receiver<u32>,
    mut rx_saved_te_longterm_webservice : Receiver<bool>,
)
{
    // temp tiles
    let regions = crate::map::get_region_ids(2);
    let mut arc_regions = HashMap::<TetrahedronId, Arc<Mutex<TempMapBuffer>>>::new();

    for id in regions.into_iter()
    {
        println!("webservice -preload region {}", id.to_string());
        arc_regions.insert(id.clone(), Arc::new(Mutex::new(TempMapBuffer::new())));
    }
    println!("len on preload {}",arc_regions.len());

    let regions_adder_reference = Arc::new(arc_regions);
    let regions_cleaner_reference = regions_adder_reference.clone();
    let regions_reader_reference = regions_adder_reference.clone();

    let towers = (0, [0; 100000]);
    let towers_adder_reference= Arc::new(Mutex::new(towers));
    let towers_cleaner_reference = towers_adder_reference.clone();
    let towers_reader_reference = towers_adder_reference.clone();

    // let chat = ChatStorage { index: 0, count: 0, record: [0; CHAT_ENTRY_SIZE * CHAT_STORAGE_SIZE] };
    let chat_adder_reference= Arc::new(Mutex::new(HashMap::new()));
    let chat_reader_reference = chat_adder_reference.clone();

    let context = AppContext 
    {
        working_game_map : working_map,
        storage_game_map : storage_map,
        server_state,
        definitions_data,
        db_client : db_client,
        cached_presentation_data: Arc::new(Mutex::new(presentation_cache)),
        temp_regions : regions_reader_reference,
        temp_towers : towers_reader_reference,
        old_messages : chat_reader_reference,
    };


    tokio::spawn(async move {
        let addr = SocketAddr::from(([0, 0, 0, 0], 3030));
        let make_service = make_service_fn(move |conn: &AddrStream| {
            let context = context.clone();
            let _addr = conn.remote_addr();
            let service = service_fn(move |req| {
                route(context.clone(), req)
            });

            // Return the service to hyper.
            async move { Ok::<_, Infallible>(service) }
        });

        // Then bind and serve...
        let server = Server::bind(&addr).serve(make_service);

        // And run forever...
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    });

    tokio::spawn(async move {
        loop {
            let message = rx_me_realtime_webservice.recv().await.unwrap();
            let region_id = message.id.get_parent(7);

            let region_map = regions_adder_reference.get(&region_id).unwrap();
            let mut region_map_lock = region_map.lock().await;


            if let Some(index) =  region_map_lock.tile_to_index.get(&message.id)
            {
                let idx = *index;
                region_map_lock.buffer[idx.. idx + MapEntity::get_size()].copy_from_slice(&message.to_bytes());
                // println!("Replaced temp data to regions map {}", idx);
            }
            else {
                let index = region_map_lock.index;
                if index + MapEntity::get_size() < region_map_lock.buffer.len() {
                    // println!("index {}, size {}", index, MapEntity::get_size());
                    
                    region_map_lock.buffer[index .. index + MapEntity::get_size()].copy_from_slice(&message.to_bytes());
                    region_map_lock.index = index + MapEntity::get_size();

                    region_map_lock.tile_to_index.insert(message.id.clone(), index);
                }
                else {
                    println!("webservice - temp buffer at capacity {}", region_id);
                }
            }
        }
    });

    tokio::spawn(async move {
        let regions = crate::map::get_region_ids(2);
        loop 
        {
            let _message = rx_saved_me_longterm_webservice.recv().await.unwrap();

            for region_id in &regions
            {
                let region_map = regions_cleaner_reference.get(&region_id).unwrap();
                let mut region_map_lock = region_map.lock().await;
                region_map_lock.index = 0;
                region_map_lock.tile_to_index.clear();
            }
        }
    });


    // towers
    tokio::spawn(async move 
    {
        loop 
        {
            let message = rx_te_realtime_webservice.recv().await.unwrap();
            let mut towers = towers_adder_reference.lock().await;
            let index = towers.0;
            if index + TOWER_ENTITY_SIZE < 100000 
            {
                towers.1[index .. index + TOWER_ENTITY_SIZE].copy_from_slice(&message.to_bytes());
                towers.0 = index + TOWER_ENTITY_SIZE;
            }
            else
            {
                println!("tower temp buffer max size reached");
            }
        }
    });

    tokio::spawn(async move {
        loop 
        {
            // a message here means that all towers have been saved to disk.
            let _message = rx_saved_te_longterm_webservice.recv().await.unwrap();

            let mut towers = towers_cleaner_reference.lock().await;
            towers.0 = 0;
        }
    });

    // we keep the last 20 messages just for fun
    tokio::spawn(async move {
        loop 
        {
            let message = rx_ce_realtime_webservice.recv().await.unwrap();


            let message_bytes = message.to_bytes();
            let mut chat = chat_adder_reference.lock().await;

            let message_faction = message.faction;
            if !chat.contains_key(&message_faction)
            {
                let new_storage = ChatStorage { index: 0, count: 0, record: [0; CHAT_ENTRY_SIZE * CHAT_STORAGE_SIZE] };
                chat.insert(message_faction, new_storage);
            }

            if let Some(messages) = chat.get_mut(&message_faction)
            {
                let index = messages.index;
                println!("chat index {index} {}", messages.count);
                messages.count = usize::min(CHAT_STORAGE_SIZE, messages.count + 1);
                messages.index = (index + 1) % CHAT_STORAGE_SIZE;
                let offset = index * CHAT_ENTRY_SIZE;
                messages.record[offset..offset + CHAT_ENTRY_SIZE].copy_from_slice(&message_bytes);
                println!("index {}", index)
            }
        }
    });
}