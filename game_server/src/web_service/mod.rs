
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64};
use std::io::Write;
use std::{sync::Arc};

use bson::oid::ObjectId;
use futures_util::lock::Mutex;
use hyper::http::Error;
use hyper::{Request, body, server::conn::AddrStream};
use hyper_static::serve::ErrorKind;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Sender, Receiver};

use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use futures_util::stream::StreamExt;

use crate::long_term_storage_service::db_character::StoredCharacter;
use crate::long_term_storage_service::db_player::StoredPlayer;
use crate::long_term_storage_service::db_region::StoredRegion;
use crate::long_term_storage_service::db_world::StoredWorld;
use crate::map::{GameMap};
use crate::map::map_entity::{MapCommand, MapEntity, MapCommandInfo};
use crate::map::tetrahedron_id::TetrahedronId;
use crate::character::character_entity::{CharacterEntity, InventoryItem};
use crate::character::character_presentation::CharacterPresentation;


#[derive(Deserialize, Serialize, Debug)]
struct PlayerCreationRequest {
    player_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct PlayerCreationResponse {
    player_token: String
}

#[derive(Deserialize, Serialize, Debug)]
struct CharacterCreationRequest {
    player_token: String,
    character_name:String,
    faction:String,
}

#[derive(Deserialize, Serialize, Debug)]
struct CharacterCreationResponse {
    character_id:u16,
}

#[derive(Deserialize, Serialize, Debug)]
struct PlayerDetailsRequest {
    player_name:String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Iteration {
    playing:bool,
    world_name:String,
    character_id:u16,
    character_name:String
}

#[derive(Deserialize, Serialize, Debug)]
struct PlayerDetailsResponse {
    player_token: String,
    joined_worlds: Vec<Iteration>,
    active_worlds: Vec<String>
}

#[derive(Deserialize, Serialize, Debug)]
struct JoinWithCharacterRequest {
    player_token: String,
    character_id:u16,
}

#[derive(Deserialize, Serialize, Debug)]
struct JoinWithCharacterResponse {
    session_id:u64,
    character_id:u16,
    faction:u8,
    tetrahedron_id:String,
    position:[f32;3],
    health:u16,
    constitution:u16
}

#[derive(Deserialize, Serialize, Debug)]
struct SellItemRequest {
    player_token: String,
    character_id:u16,
    old_item_id:u32,
    amount:u16,
}

#[derive(Deserialize, Serialize, Debug)]
struct SellItemResponse {
    new_item_id:u32,
    amount:u16,
}

#[derive(Clone)]
struct AppContext {
    last_presentation_update: Arc<AtomicU64>,
    presentation_data: Arc<Mutex<HashMap<u16, [u8;22]>>>,
    compressed_presentation_data: Arc<Mutex<Vec<u8>>>,// we will keep a copy and update it more or less frequently.
    working_game_map : Arc<GameMap>,
    storage_game_map : Arc<GameMap>,
    tx_mc_webservice_realtime : Sender<MapCommand>,
    db_client : mongodb ::Client,
    temp_regions : Arc::<HashMap::<TetrahedronId, Arc<Mutex<TempMapBuffer>>>>
}

// async fn handle_update_map_entity(context: AppContext, mut req: Request<Body>) -> Result<Response<Body>, hyper::http::Error> {

//     let body = req.body_mut();
//     let data = body::to_bytes(body).await.unwrap();
//     let data: PlayerRequest = serde_json::from_slice(&data).unwrap();
//     println!("handling request {:?}", data);
//     let tile_id = TetrahedronId::from_string(&data.tile_id);
//     let region = context.working_game_map.get_region_from_child(&tile_id);
//     let mut tiles = region.lock().await;
//     let tile_data = tiles.get_mut(&tile_id);

//     match tile_data {
//         Some(tile_data) => {

//             let tile = MapEntity{
//                 object_id: tile_data.object_id,
//                 version: tile_data.version,
//                 id: tile_data.id.clone(),

//                 owner_id: tile_data.owner_id,
//                 ownership_time: tile_data.ownership_time,

//                 origin_id: tile_data.origin_id.clone(),
//                 target_id: tile_data.target_id.clone(),
//                 time: 0,
//                 prop: data.prop,
//                 faction : tile_data.faction,
//                 level : tile_data.level,
//                 temperature : tile_data.temperature,
//                 moisture : tile_data.moisture,
//                 heights : tile_data.heights,
//                 pathness : tile_data.pathness,
//                 health: tile_data.health,
//                 constitution: tile_data.constitution,
//             };

//             let player_response = PlayerResponse {
//                 tile_id :tile_id.to_string(),
//                 success : format!("tile updated with {}", tile.prop)
//             };

//             *tile_data = tile;

//             let map_command = MapCommand {
//                 id : tile_data.id.clone(),
//                 info : MapCommandInfo::Touch()
//             };

//             let _ = context.tx_mc_webservice_realtime.send(map_command).await;


//             let response = serde_json::to_vec(&player_response).unwrap();
//             Ok(Response::new(Body::from(response)))
//         },
//         None => {

//             let player_response = PlayerResponse {
//                 tile_id :tile_id.to_string(),
//                 success : "tile doesn't exist".to_owned()
//             };
//             let response = serde_json::to_vec(&player_response).unwrap();
//             Ok(Response::new(Body::from(response)))
//         }
//     }
// }

async fn handle_create_player(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> {

    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: PlayerCreationRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);

    let data_collection: mongodb::Collection<StoredPlayer> = context.db_client.database("game").collection::<StoredPlayer>("players");
    let data_from_db: Option<StoredPlayer> = data_collection
    .find_one(
        bson::doc! {
                "player_name": data.player_name.clone(),
        },
        None,
    ).await
    .unwrap();


    if let Some(_player) = data_from_db 
    {
        let mut response = Response::new(Body::from(String::from("Player already created")));
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
    }

    let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
    let player_token = format!("token_{}", current_time.as_secs());
    println!("got a {} as player token",player_token);

    let stored_character = StoredPlayer{
        id: None,
        player_name: data.player_name,
        player_token: player_token.clone(),
    };

    let result = data_collection.insert_one(stored_character, None).await.unwrap();

    let object_id: Option<ObjectId> = match result.inserted_id {
        bson::Bson::ObjectId(id) => Some(id),
        _ => None,
    };

    let new_character = PlayerCreationResponse{
        player_token,
    };

    let response = serde_json::to_vec(&new_character).unwrap();
    Ok(Response::new(Body::from(response)))
}


async fn handle_player_request(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> {

    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: PlayerDetailsRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);

    let data_collection: mongodb::Collection<StoredPlayer> = context.db_client.database("game").collection::<StoredPlayer>("players");
    let stored_player: Option<StoredPlayer> = data_collection
    .find_one(
        bson::doc! {
                "player_name": data.player_name,
        },
        None,
    ).await
    .unwrap();

    if stored_player.is_none()
    {
        let mut response = Response::new(Body::from(String::from("Player not found")));
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
    }

    let player_data = stored_player.unwrap();
    let stored_player_id = player_data.id;

    let data_collection: mongodb::Collection<StoredCharacter> = context.db_client.database("game").collection::<StoredCharacter>("characters");
    let mut characters_cursor = data_collection
    .find(
        bson::doc! {
                "player_id": stored_player_id,
        },
        None,
    ).await
    .unwrap();


    let mut characters : Vec<Iteration>= Vec::new();
    while let Some(result) = characters_cursor.next().await {
        match result 
        {
            Ok(doc) => {
                characters.push(Iteration {
                    playing: true, 
                    world_name: doc.world_name.clone(),
                    character_id: doc.character_id,
                    character_name: doc.character_name.to_string(),
                    })
            },
            Err(error_details) => {
                println!("error getting characters from db with {:?}", error_details);
            },
        }
    }
    println!("----- characters {}", characters.len());

    // now get all the worlds to see available ones

    let worlds_collection: mongodb::Collection<StoredWorld> = context.db_client.database("game").collection::<StoredWorld>("worlds");

    let mut active_worlds = Vec::<String>::new();
    //  I should be requesting all available worlds...
    let data_from_db = worlds_collection
    .find_one(
        bson::doc! {
                "world_name": context.storage_game_map.world_name.to_owned()
        },
        None,
    ).await.unwrap();

    if let Some(w) = data_from_db
    {
        active_worlds.push(w.world_name);
    }

    let response_data = PlayerDetailsResponse {
        
        joined_worlds: characters,
        active_worlds,
        player_token: player_data.player_token,
    };

    let response_json = serde_json::to_vec(&response_data).unwrap();
    let response = Response::new(Body::from(response_json));
    return Ok(response);
}

async fn handle_create_character(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> {

    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: CharacterCreationRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);

    let data_collection: mongodb::Collection<StoredPlayer> = context.db_client.database("game").collection::<StoredPlayer>("players");
    let data_from_db: Option<StoredPlayer> = data_collection
    .find_one(
        bson::doc! {
                "player_token": data.player_token,
        },
        None,
    ).await
    .unwrap();

    let player_id = data_from_db.map(|p| p.id).flatten();

    if player_id.is_none()
    {
        let mut response = Response::new(Body::from(String::from("player doesn't exist")));
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
    }

    let generator = &context.working_game_map.id_generator;
    let new_id = generator.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    println!("got a {} as id base ",new_id);

    let stored_character = StoredCharacter{
        id: None,
        player_id,
        version:1,
        world_id: context.working_game_map.world_id.clone(),
        world_name: context.working_game_map.world_name.clone(),
        character_id: new_id,
        character_name: data.character_name.clone(),
        position:[0f32,0f32,0f32],
        faction: data.faction.clone(),
        inventory : Vec::new(),
        constitution: 100,
        health: 100,
        attack: 5,
        defense: 5,
        agility: 5,
    };

    let data_collection: mongodb::Collection<StoredCharacter> = context.db_client.database("game").collection::<StoredCharacter>("characters");
    let result = data_collection.insert_one(stored_character, None).await.unwrap();

    let object_id: Option<ObjectId> = match result.inserted_id {
        bson::Bson::ObjectId(id) => Some(id),
        _ => None,
    };

    let player_entity = CharacterEntity {
        object_id: object_id,
        player_id,
        character_name : data.character_name.clone(),
        character_id: new_id,
        version:1,
        faction: CharacterEntity::get_faction_code(&data.faction),
        action: 0,
        position: [0.0, 0.0, 0.0],
        second_position: [0.0, 0.0, 0.0],
        constitution: 100,
        health: 100,
        inventory: Vec::new(), // fill this from storedcharacter
        inventory_hash : 1,
        attack: 5,
        defense: 5,
        agility: 5,
    };

    let mut players = context.working_game_map.players.lock().await;
    players.insert(new_id, player_entity.clone());
    drop(players);

    let mut players = context.storage_game_map.players.lock().await;
    players.insert(new_id, player_entity);

    drop(players);

    let new_character = CharacterCreationResponse{
        character_id: new_id,
    };

    let response = serde_json::to_vec(&new_character).unwrap();
    Ok(Response::new(Body::from(response)))
}

async fn handle_login_character(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> {

    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: JoinWithCharacterRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);

    let data_collection: mongodb::Collection<StoredPlayer> = context.db_client.database("game").collection::<StoredPlayer>("players");
    let data_from_db: Option<StoredPlayer> = data_collection
    .find_one(
        bson::doc! {
                "player_token": data.player_token.clone(),
        },
        None,
    ).await
    .unwrap();

    let is_valid = if let Some(player) = data_from_db 
    {
        player.player_token == data.player_token
    }
    else {
        false 
    };

    if !is_valid 
    {
        let mut response = Response::new(Body::from(String::from("player token is not valid")));
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
    }

    let players = context.working_game_map.players.lock().await;

    if let Some(player) = players.get(&data.character_id) {
        println!("player login {:?}", player);
        let mut presentation_map = context.presentation_data.lock().await;
        let name_with_padding = format!("{: <5}", player.character_name);
        let name_data : Vec<u32> = name_with_padding.chars().into_iter().map(|c| c as u32).collect();
        let mut name_array = [0u32; 5];
        name_array.clone_from_slice(&name_data.as_slice()[0..5]);

        let player_presentation = CharacterPresentation {
            player_id: player.character_id,
            character_name: name_array,
        };

        presentation_map.insert(data.character_id, player_presentation.to_bytes());
        println!("position {:?} {}", player.position, player.health);


        let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
        let session_id = current_time.as_secs();

        let saved_char = JoinWithCharacterResponse{
            character_id: player.character_id,
            faction:player.faction,
            tetrahedron_id:"k120223211".to_owned(),
            position: player.second_position,
            health: player.health,
            constitution: player.constitution,
            session_id,
        };
        println!("creating session id {} for {}", session_id ,player.character_id);
        let session_option = context.working_game_map.logged_in_players.get(&player.character_id);
        if let Some(session) = session_option {
            session.store(session_id, std::sync::atomic::Ordering::Relaxed);
        }

        let response = serde_json::to_vec(&saved_char).unwrap();
        Ok(Response::new(Body::from(response)))
    }
    else {
        Ok(Response::new(Body::from("error: char not found")))
    }

    
}

async fn handle_sell_item(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, Error> {

    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: SellItemRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);

    let mut players = context.working_game_map.players.lock().await;

    if let Some(player) = players.get_mut(&data.character_id) {
        // println!("selling player {:?}", player);

        // let mut updated_player_entity = player.clone();
        let result = player.remove_inventory_item(InventoryItem{
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

async fn handle_region_request(context: AppContext, data : Vec<&str>) -> Result<Response<Body>, Error> {

    let mut iterator = data.into_iter();
    let region_list = iterator.next();
    let regions = if let Some(regions_csv) = region_list {
        println!("{}", regions_csv);
        let data = regions_csv.split(",");
        let regions_ids : Vec<&str> = data.collect();
        let iterator : Vec<TetrahedronId> = regions_ids.into_iter().map(|id| TetrahedronId::from_string(id)).collect();
        iterator
    } else {
        Vec::new()
    };

    let mut binary_data = Vec::<u8>::new();
    let mut stored_regions_data = Vec::<Vec<u8>>::new();
    for region_id in &regions 
    {
        let data_collection: mongodb::Collection<StoredRegion> = context.db_client.database("game").collection::<StoredRegion>("regions");

        // Look up one document:
        let data_from_db: Option<StoredRegion> = data_collection
        .find_one(
            bson::doc! {
                    "world_id": context.storage_game_map.world_id,
                    "region_id": region_id.to_string()
            },
            None,
        ).await
        .unwrap();

        if let Some(region_from_db) = data_from_db {
            println!("region id {:?} with version {}", region_from_db.region_id, region_from_db.region_version);
            let region_data: Vec<u8> = match region_from_db.compressed_data {
                bson::Bson::Binary(binary) => binary.bytes,
                _ => panic!("Expected Bson::Binary"),
            };
            stored_regions_data.push(region_data);
        }
        else {
            stored_regions_data.push(Vec::new());
        }
    }

    for region_data in &stored_regions_data{
        let size_bytes = u32::to_le_bytes(region_data.len() as u32);
        binary_data.extend_from_slice(&size_bytes);
    }

    for region_data in &mut stored_regions_data
    {
        binary_data.append(region_data);
    }

    let response = Response::builder()
        .status(hyper::StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(binary_data))
        .expect("Failed to create response");
    Ok(response)
}

async fn handle_temp_region_request(context: AppContext, data : Vec<&str>) -> Result<Response<Body>, Error> {

    let mut iterator = data.into_iter();
    let region_list = iterator.next();

// this string might contain more than one region separated by semicolon
    let regions = if let Some(regions_csv) = region_list {
        println!("{}", regions_csv);
        let data = regions_csv.split(",");
        let regions_ids : Vec<&str> = data.collect();
        let iterator : Vec<TetrahedronId> = regions_ids.into_iter().map(|id| TetrahedronId::from_string(id)).collect();
        iterator
    } else {
        Vec::new()
    };

    let mut binary_data = Vec::<u8>::new();
    for region_id in &regions 
    {
        let region_map = context.temp_regions.get(region_id).unwrap();
        let region_map_lock = region_map.lock().await;
        let size = region_map_lock.index;
        binary_data.extend_from_slice(&region_map_lock.buffer[..size]);
    }

    let response = Response::builder()
        .status(hyper::StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .body(Body::from(binary_data))
        .expect("Failed to create response");
    Ok(response)
}

async fn handle_characters_request(context: AppContext) -> Result<Response<Body>, Error> {

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));
    
    let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
    let last_time_atomic = context.last_presentation_update;
    let last_time = last_time_atomic.load(std::sync::atomic::Ordering::Relaxed);
    if current_time.as_secs() - last_time > 10 {

        println!("generating presentation data {}", last_time);
        let players_presentation_map = context.presentation_data.lock().await;

        for presentation_data in players_presentation_map.iter()
        {
            let bytes = presentation_data.1;
            encoder.write_all(bytes).unwrap();
        }

        drop(players_presentation_map);
        let mut compressed_bytes = encoder.reset(Vec::new()).unwrap();
        let mut presentation_cache = context.compressed_presentation_data.lock().await;

        presentation_cache.clear();
        presentation_cache.append(&mut compressed_bytes);

        last_time_atomic.store(current_time.as_secs(), std::sync::atomic::Ordering::Relaxed);

        let response = Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "application/octet-stream")
            .body(Body::from(presentation_cache.clone()))
            .expect("Failed to create response");
        Ok(response)
    }
    else
    {
        println!("using cache {}", last_time);
        let presentation_cache = context.compressed_presentation_data.lock().await;

        let response = Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "application/octet-stream")
            .body(Body::from(presentation_cache.clone()))
            .expect("Failed to create response");
        Ok(response)

    }
}

async fn route(context: AppContext, req: Request<Body>) -> Result<Response<Body>, Error> {

    let uri = req.uri().to_string();
    let mut data = uri.split("/");
    data.next();
    if let Some(route) = data.next() {
        let rest : Vec<&str> = data.collect();
        match route {
            "region" => handle_region_request(context, rest).await,
            "temp_regions" => handle_temp_region_request(context, rest).await,
            "character_data" => handle_characters_request(context).await,
            "player_creation" => handle_create_player(context, req).await,
            "player_data" => handle_player_request(context, req).await,
            "character_creation" => handle_create_character(context, req).await,
            "join_with_character" => handle_login_character(context, req).await,
            "sell_item" => handle_sell_item(context, req).await,
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
    working_map: Arc<GameMap>, 
    storage_map: Arc<GameMap>, 
    db_client : mongodb :: Client,
    mut rx_me_realtime_webservice : Receiver<MapEntity>,
    tx_mc_webservice_gameplay : Sender<MapCommand>,
    mut rx_saved_longterm_webservice : Receiver<u32>)
    {

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


    let context = AppContext {
        presentation_data : Arc::new(Mutex::new(HashMap::new())),
        working_game_map : working_map,
        storage_game_map : storage_map,
        tx_mc_webservice_realtime : tx_mc_webservice_gameplay,
        db_client : db_client,
        last_presentation_update: Arc::new(AtomicU64::new(0)),
        compressed_presentation_data: Arc::new(Mutex::new(Vec::new())),
        temp_regions : regions_reader_reference
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
        loop {
            let _message = rx_saved_longterm_webservice.recv().await.unwrap();

            for region_id in &regions
            {
                let region_map = regions_cleaner_reference.get(&region_id).unwrap();
                let mut region_map_lock = region_map.lock().await;
                region_map_lock.index = 0;
                region_map_lock.tile_to_index.clear();
            }
        }
    });
}