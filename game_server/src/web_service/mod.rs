
use std::collections::HashMap;
use std::sync::Arc;

use futures_util::future::OrElse;
use futures_util::lock::Mutex;
use hyper::http::Error;
use hyper::{Request, body, server::conn::AddrStream};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;

use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};

use crate::hero::hero_inventory::InventoryItem;
use crate::chat::chat_entry::{ChatEntry, CHAT_ENTRY_SIZE};
use crate::definitions::definitions_container::DefinitionsData;
use crate::kingdom::kingdom_entity::{KingdomEntity, KINGDOM_ENTITY_SIZE};
use crate::map::GameMap;
use crate::map::map_entity::MapEntity;
use crate::map::tetrahedron_id::TetrahedronId;
use crate::mob::mob_entity::{MobEntity, MOB_ENTITY_SIZE};
use crate::ServerState;
use crate::tower::tower_entity::{TowerEntity, TOWER_ENTITY_SIZE};

pub mod heroes;
pub mod map;
pub mod towers;
pub mod kingdoms;
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
    temp_mobs_regions : Arc::<HashMap::<TetrahedronId, Arc<Mutex<TempMobBuffer>>>>,
    temp_towers : Arc::<Mutex::<(usize,[u8;100000])>>,
    temp_kingdoms : Arc::<Mutex::<(usize,[u8;10000])>>,
    old_messages : Arc::<Mutex::<HashMap<u8, ChatStorage>>>//index, offset, count, 20 messages
}

pub fn create_response_builder() -> hyper::http::response::Builder
{
    let response = Response::builder()
        .status(hyper::StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        .header("Access-Control-Allow-Headers", "*");
    response
}

// deprecated, moved to the character protocol to avoid abusing form the web server.
async fn handle_sell_item(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> 
{
    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: SellItemRequest = serde_json::from_slice(&data).unwrap();
    cli_log::info!("handling request {:?}", data);

    let mut players = context.working_game_map.character.lock().await;

    if let Some(player) = players.get_mut(&data.character_id) {
        // cli_log::info!("selling player {:?}", player);

        // let mut updated_player_entity = player.clone();
        let result = player.remove_inventory_item(InventoryItem
        {
            item_id: data.old_item_id,
            equipped: 0,
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
            equipped: 0,
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

// async fn handle_check_version(_context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> 
// {
//     let body = req.body_mut();
//     let data = body::to_bytes(body).await.unwrap();
//     cli_log::info!("handling request {:?}", data);
//     let data: ClientVersionRequest = serde_json::from_slice(&data).unwrap();

//     let response = ClientVersionResponse
//     {
//         server_version : 5
//     };

//     let data = serde_json::to_vec(&response).unwrap();

//     let response = create_response_builder()
//         .body(Body::from(data))
//         .expect("Failed to create response");
//     Ok(response)
// }

async fn handle_check_version(_context: AppContext, mut req: Request<Body>) ->Result<Body, String> 
{
    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    cli_log::info!("handling request {:?}", data);
    let _data: ClientVersionRequest = serde_json::from_slice(&data).unwrap();

    let response = ClientVersionResponse
    {
        server_version : 5
    };

    let data = serde_json::to_vec(&response).unwrap();

    Ok(Body::from(data))
}


async fn handle_definition_request(context: AppContext, mut req: Request<Body>) ->Result<Body, String> 
{
    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let result : Result<DefinitionRequest, _> = serde_json::from_slice(&data);
    if let Ok(data) = result
    {
        cli_log::info!("handling request {:?}", data);

        let data = if let Some(definition_data)= context.definitions_data.definition_versions.get(&data.name)
        {
            if definition_data.version == data.version && data.name == "character_progression"
            {
                Some(context.definitions_data.character_progression_data)
            }
            else if definition_data.version == data.version && data.name == "definition_versions"
            {
                Some(context.definitions_data.definition_versions_data)
            }
            else if definition_data.version == data.version && data.name == "props"
            {
                Some(context.definitions_data.props_data)
            }
            else if definition_data.version == data.version && data.name == "mob_progression"
            {
                Some(context.definitions_data.mob_progression_data)
            }
            else if definition_data.version == data.version && data.name == "main_paths"
            {
                Some(context.definitions_data.main_paths_data)
            }
            else if definition_data.version == data.version && data.name == "items"
            {
                Some(context.definitions_data.items_data)
            }
            else if definition_data.version == data.version && data.name == "cards"
            {
                Some(context.definitions_data.cards_data)
            }
            else if definition_data.version == data.version && data.name == "mobs"
            {
                Some(context.definitions_data.mobs_data)
            }
            else if definition_data.version == data.version && data.name == "buffs"
            {
                Some(context.definitions_data.buffs_data)
            }
            else if definition_data.version == data.version && data.name == "weapons"
            {
                Some(context.definitions_data.weapons_data)
            }
            else if definition_data.version == data.version && data.name == "towers_difficulty"
            {
                Some(context.definitions_data.towers_difficulty_data)
            }
            else
            {
                None
            }
        }
        else
        {
            None
        };

        if let Some(data) = data 
        {
            Ok(Body::from(data))
        }
        else 
        {
            return Err("definition_not_found".to_owned());
        }
    }
    else
    {
        return Err("request_error".to_owned());
    }

}

async fn route(context: AppContext, req: Request<Body>) -> Result<Response<Body>, Error> 
{
    let uri = req.uri().to_string();
    let mut data = uri.split("/");
    data.next();
    if let Some(route) = data.next() 
    {
        let rest : Vec<&str> = data.collect();

        let builder = create_response_builder();

        // Handle preflight requests
        if req.method() == hyper::Method::OPTIONS 
        {
            return Ok(builder.status(StatusCode::NO_CONTENT).body(Body::empty()).unwrap());
        }

        let body = match route 
        {
            "region" => map::handle_region_request(context, rest).await,
            "temp_regions" => map::handle_temp_region_request(context, rest).await,
            "temp_mob_regions" => map::handle_temp_mob_region_request(context, rest).await,
            "definitions" => handle_definition_request(context, req).await,
            "character_data" => heroes::handle_characters_request(context).await,
            "player_creation" => heroes::handle_create_player(context, req).await,
            "player_data" => heroes::handle_player_request(context, req).await,
            "hero_creation" => heroes::handle_create_hero(context, req).await,
            "join_with_hero" => heroes::handle_login_with_hero(context, req).await,
            "towers" => towers::handle_request_towers(context, req).await,
            "temp_towers" => towers::handle_temp_tower_request(context).await,
            "kingdoms" => kingdoms::handle_request_kingdoms(context, req).await,
            "temp_kingdoms" => kingdoms::handle_temp_kingdoms_request(context).await,
            // // "sell_item" => handle_sell_item(context, req).await,
            "chat_record" => chat::handle_chat_record_request(context, rest).await,
            "exchange_skill_points" => heroes::exchange_skill_points(context, req).await,
            "check_version" => handle_check_version(context, req).await,
            _ => 
            {
                cli_log::warn!("route not found: {route}");
                let mut response = create_response_builder().body(Body::from("route_not_found")).unwrap();
                *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
                return Ok(response);
            }
        };

        match body
        {
            Ok(body) => 
            {
                let response = create_response_builder().body(body);
                response
            }
            Err(error) => 
            {
                let mut response = create_response_builder().body(Body::from(error)).unwrap();
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                Ok(response)
            },
        }
    }
    else 
    {
        let mut response = Response::new(Body::from(String::from("missing route")));
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
    }
}


pub struct TempMapBuffer 
{ 
    pub index : usize,
    pub tile_to_index : HashMap<TetrahedronId, usize>,
    pub defeated_mob_and_time: HashMap<TetrahedronId, u32>,
    pub buffer : [u8;100000]
}

impl TempMapBuffer 
{
    pub fn new() -> TempMapBuffer
    {
        TempMapBuffer { index: 0, buffer: [0; 100000], tile_to_index : HashMap::new(), defeated_mob_and_time : HashMap::new() }
    }
}

pub struct TempMobBuffer 
{ 
    pub index : usize,
    pub mob_id_to_index : HashMap<u32, usize>,
    pub defeated_mob_and_time: HashMap<u32, u32>,
    pub buffer : [u8;100000]
}

impl TempMobBuffer 
{
    pub fn new() -> TempMobBuffer
    {
        TempMobBuffer { index: 0, buffer: [0; 100000], mob_id_to_index : HashMap::new(), defeated_mob_and_time : HashMap::new() }
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
    mut rx_moe_realtime_webservice : Receiver<MobEntity>,
    mut rx_te_realtime_webservice : Receiver<TowerEntity>,
    mut rx_ke_realtime_webservice : Receiver<KingdomEntity>,
    mut rx_ce_realtime_webservice : Receiver<ChatEntry>,
    // tx_mc_webservice_gameplay : Sender<MapCommand>,
    mut rx_saved_me_longterm_webservice : Receiver<u32>,
    mut rx_saved_te_longterm_webservice : Receiver<bool>,
    mut rx_saved_ke_longterm_webservice : Receiver<bool>,
)
{
    // temp tiles
    let regions = crate::map::get_region_ids(2);
    let mut arc_regions = HashMap::<TetrahedronId, Arc<Mutex<TempMapBuffer>>>::new();

    for id in regions.iter()
    {
        cli_log::info!("webservice -preload map region {}", id.to_string());
        arc_regions.insert(id.clone(), Arc::new(Mutex::new(TempMapBuffer::new())));
    }
    cli_log::info!("mpa regions len on preload {}",arc_regions.len());

    let regions_adder_reference = Arc::new(arc_regions);
    let regions_cleaner_reference = regions_adder_reference.clone();
    let regions_reader_reference = regions_adder_reference.clone();

    // temp mobs
    let mut arc_mob_regions = HashMap::<TetrahedronId, Arc<Mutex<TempMobBuffer>>>::new();

    for id in regions.into_iter()
    {
        cli_log::info!("webservice -preload mob region {}", id.to_string());
        arc_mob_regions.insert(id, Arc::new(Mutex::new(TempMobBuffer::new())));
    }
    cli_log::info!("mob regions len on preload {}",arc_mob_regions.len());

    let mob_regions_adder_reference = Arc::new(arc_mob_regions);
    // this is not saved to disk, we never delete... haha... 
    // let mob_regions_cleaner_reference = mob_regions_adder_reference.clone();
    let mob_regions_reader_reference = mob_regions_adder_reference.clone();

    // towers

    let towers = (0, [0; 100000]);
    let towers_adder_reference= Arc::new(Mutex::new(towers));
    let towers_cleaner_reference = towers_adder_reference.clone();
    let towers_reader_reference = towers_adder_reference.clone();

    let kingdoms = (0, [0; 10000]);
    let kingdoms_adder_reference= Arc::new(Mutex::new(kingdoms));
    let kingdoms_cleaner_reference = kingdoms_adder_reference.clone();
    let kingdoms_reader_reference = kingdoms_adder_reference.clone();

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
        temp_mobs_regions : mob_regions_reader_reference,
        temp_towers : towers_reader_reference,
        temp_kingdoms : kingdoms_reader_reference,
        old_messages : chat_reader_reference,
    };


    tokio::spawn(async move 
    {
        let addr = SocketAddr::from(([0, 0, 0, 0], 3031));
        let make_service = make_service_fn(move |conn: &AddrStream| 
        {
            let context = context.clone();
            let _addr = conn.remote_addr();
            let service = service_fn(move |req| 
            {
                route(context.clone(), req)
            });

            // Return the service to hyper.
            async move { Ok::<_, Infallible>(service) }
        });

        // Then bind and serve...
        let server = Server::bind(&addr).serve(make_service);

        // And run forever...
        if let Err(e) = server.await 
        {
            cli_log::info!("server error: {}", e);
        }
    });

    // saves map entities 
    tokio::spawn(async move 
    {
        loop 
        {
            let message = rx_me_realtime_webservice.recv().await.unwrap();
            let region_id = message.id.get_parent(7);

            let region_map = regions_adder_reference.get(&region_id).unwrap();
            let mut region_map_lock = region_map.lock().await;


            if let Some(index) =  region_map_lock.tile_to_index.get(&message.id)
            {
                let idx = *index;
                region_map_lock.buffer[idx.. idx + MapEntity::get_size()].copy_from_slice(&message.to_bytes());
                // cli_log::info!("Replaced temp data to regions map {}", idx);
            }
            else {
                let index = region_map_lock.index;
                if index + MapEntity::get_size() < region_map_lock.buffer.len() {
                    // cli_log::info!("index {}, size {}", index, MapEntity::get_size());
                    
                    region_map_lock.buffer[index .. index + MapEntity::get_size()].copy_from_slice(&message.to_bytes());
                    region_map_lock.index = index + MapEntity::get_size();

                    region_map_lock.tile_to_index.insert(message.id.clone(), index);
                }
                else {
                    cli_log::info!("webservice - temp buffer at capacity {}", region_id);
                }
            }
        }
    });

    // clears map entities from the cache
    tokio::spawn(async move 
    {
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


    fn remove_mob_at_index(to_be_removed : u32, mob_region_map_lock : &mut TempMobBuffer)
    {
        if let Some(index) = mob_region_map_lock.mob_id_to_index.get(&to_be_removed).map(|s| *s)
        {
            let current_index = mob_region_map_lock.index;
            // here we are mvoing the last one to the empty spot!, this way the list wont dumbly grow.
            if current_index == 0
            {
                mob_region_map_lock.mob_id_to_index.remove(&to_be_removed);
            }
            else if index == current_index - MobEntity::get_size()
            {
                mob_region_map_lock.index = index;
                mob_region_map_lock.mob_id_to_index.remove(&to_be_removed);
            }
            else
            {
                let last_index = current_index - MobEntity::get_size();
                let last_entry = &mob_region_map_lock.buffer[last_index..last_index + MobEntity::get_size()]; // last one

                let mob_id = u32::from_le_bytes(last_entry[0..4].try_into().unwrap()); // 4 bytes

                let mut buffer = [0u8;MOB_ENTITY_SIZE];
                buffer.copy_from_slice(last_entry);

                mob_region_map_lock.buffer[index.. index + MobEntity::get_size()].copy_from_slice(&buffer);
                mob_region_map_lock.mob_id_to_index.remove(&mob_id);
                mob_region_map_lock.mob_id_to_index.remove(&to_be_removed);
                mob_region_map_lock.defeated_mob_and_time.remove(&to_be_removed);
                mob_region_map_lock.mob_id_to_index.insert(mob_id, index);
                mob_region_map_lock.index = last_index;
                // cli_log::info!("-- replacing mob index {}", mob_region_map_lock.index);
            }
        }
    }

// saves mobs to cache, if health is 0 the mob is registered to be removed later.
tokio::spawn(async move 
{
    loop 
    {
        let message = rx_moe_realtime_webservice.recv().await.unwrap();
        let end_region_id = message.end_position_id.get_parent(7);
        let origin_region_id = message.start_position_id.get_parent(7);
        
        // mob moved to another area, we need to do crazy stuff
        // you could have stayed in your #$%#$% area, but nooo, you wanted to go further....
        if end_region_id != origin_region_id
        {
            let mob_region_map = mob_regions_adder_reference.get(&origin_region_id).unwrap();
            let mut mob_region_map_lock = mob_region_map.lock().await;
            remove_mob_at_index(message.mob_id, &mut mob_region_map_lock);
        }

        let mob_region_map = mob_regions_adder_reference.get(&end_region_id).unwrap();
        let mut mob_region_map_lock = mob_region_map.lock().await;

        if let Some(index) = mob_region_map_lock.mob_id_to_index.get(&message.mob_id).map(|s| *s)
        {
            if message.health <= 0
            {
                mob_region_map_lock.defeated_mob_and_time.insert(message.mob_id, message.ownership_time);
            }
            else
            {
                let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
                let current_time_in_millis = current_time.as_millis() as u64;
                let current_time_in_seconds = (current_time_in_millis / 1000) as u32;

                // lets find the first mob we can actually remove before adding more items
                for defeated_mob in &mob_region_map_lock.defeated_mob_and_time
                {
                    //16 minutes before despawn
                    if *defeated_mob.1  + 1000 > current_time_in_seconds
                    {
                        remove_mob_at_index(defeated_mob.0.clone(), &mut mob_region_map_lock);
                        break;
                    }
                }
            }

            let idx = index;
            let bytes = message.to_bytes();
            mob_region_map_lock.buffer[idx.. idx + MobEntity::get_size()].copy_from_slice(&bytes);
        }
        else 
        {
            if message.health > 0
            {
                let index = mob_region_map_lock.index;
                if index + MobEntity::get_size() < mob_region_map_lock.buffer.len() 
                {
                    mob_region_map_lock.buffer[index .. index + MobEntity::get_size()].copy_from_slice(&message.to_bytes());
                    mob_region_map_lock.index = index + MobEntity::get_size();
                    mob_region_map_lock.mob_id_to_index.insert(message.mob_id, index);
                    // cli_log::info!("-- updated mob index {}", mob_region_map_lock.index);
                }
                else 
                {
                    cli_log::info!("webservice - mob temp buffer at capacity {}", end_region_id);
                }
            }
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
            cli_log::info!("tower temp buffer max size reached");
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

// kingdoms
tokio::spawn(async move 
{
    loop 
    {
        let message = rx_ke_realtime_webservice.recv().await.unwrap();
        let mut kingdoms = kingdoms_adder_reference.lock().await;
        let index = kingdoms.0;
        if index + KINGDOM_ENTITY_SIZE < 10000 
        {
            kingdoms.1[index .. index + TOWER_ENTITY_SIZE].copy_from_slice(&message.to_bytes());
            kingdoms.0 = index + TOWER_ENTITY_SIZE;
        }
        else
        {
            cli_log::info!("kingdome temp buffer max size reached");
        }
    }
});


tokio::spawn(async move 
{
    loop 
    {
        // a message here means that all towers have been saved to disk.
        let _message = rx_saved_ke_longterm_webservice.recv().await.unwrap();

        let mut kingdoms = kingdoms_cleaner_reference.lock().await;
        kingdoms.0 = 0;
    }
});

// we keep the last 20 messages just for fun
tokio::spawn(async move 
{
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
            cli_log::info!("chat index {index} {}", messages.count);
            messages.count = usize::min(CHAT_STORAGE_SIZE, messages.count + 1);
            messages.index = (index + 1) % CHAT_STORAGE_SIZE;
            let offset = index * CHAT_ENTRY_SIZE;
            messages.record[offset..offset + CHAT_ENTRY_SIZE].copy_from_slice(&message_bytes);
            cli_log::info!("index {}", index)
        }
    }
});
}