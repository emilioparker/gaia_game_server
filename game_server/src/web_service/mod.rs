
use core::arch;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64};
use std::io::Write;
use std::{sync::Arc};

use bson::oid::ObjectId;
use futures_util::future::OrElse;
use futures_util::lock::Mutex;
use futures_util::stream::AbortRegistration;
use hyper::{Request, body, server::conn::AddrStream};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Sender, Receiver};

use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use flate2::Compression;
use flate2::write::ZlibEncoder;

use crate::long_term_storage_service::db_character::StoredCharacter;
use crate::long_term_storage_service::db_region::StoredRegion;
use crate::map::{GameMap};
use crate::map::map_entity::{MapCommand, MapEntity, MapCommandInfo};
use crate::map::tetrahedron_id::TetrahedronId;
use crate::player::player_entity::PlayerEntity;
use crate::player::player_presentation::PlayerPresentation;

#[derive(Deserialize, Serialize, Debug)]
struct PlayerRequest {

    tile_id: String,
    action: String, //create
    prop: u32, // tree
}

#[derive(Deserialize, Serialize, Debug)]
struct PlayerResponse {
    tile_id: String,
    success: String,
}
// character creation

#[derive(Deserialize, Serialize, Debug)]
struct CharacterCreationRequest {
    character_name: String, //create
    device_id: String
}

#[derive(Deserialize, Serialize, Debug)]
struct CharacterCreationResponse {
    character_id:u64,
}

#[derive(Deserialize, Serialize, Debug)]
struct JoinWithCharacterRequest {
    device_id: String,
    character_id:u64,
}

#[derive(Deserialize, Serialize, Debug)]
struct JoinWithCharacterResponse {
    character_id:u64,
    tetrahedron_id:String,
    health:u32,
    constitution:u32
}

#[derive(Clone)]
struct AppContext {
    last_presentation_update: Arc<AtomicU64>,
    presentation_data: Arc<Mutex<HashMap<u64, [u8;28]>>>,
    compressed_presentation_data: Arc<Mutex<Vec<u8>>>,// we will keep a copy and update it more or less frequently.
    working_game_map : Arc<GameMap>,
    storage_game_map : Arc<GameMap>,
    tx_mc_webservice_realtime : Sender<MapCommand>,
    db_client : mongodb ::Client,
    temp_regions : Arc::<HashMap::<TetrahedronId, Arc<Mutex<TempMapBuffer>>>>
}

async fn handle_update_map_entity(context: AppContext, mut req: Request<Body>) -> Result<Response<Body>, hyper::http::Error> {

    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: PlayerRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);
    let tile_id = TetrahedronId::from_string(&data.tile_id);
    let region = context.working_game_map.get_region_from_child(&tile_id);
    let mut tiles = region.lock().await;
    let tile_data = tiles.get_mut(&tile_id);

    match tile_data {
        Some(tile_data) => {

            let tile = MapEntity{
                object_id: tile_data.object_id,
                id: tile_data.id.clone(),
                last_update: tile_data.last_update,
                health: tile_data.health,
                constitution: tile_data.constitution,
                prop: data.prop,
                temperature : tile_data.temperature,
                moisture : tile_data.moisture,
                heights : tile_data.heights,
                normal_a : tile_data.normal_a,
                normal_b : tile_data.normal_b,
                normal_c : tile_data.normal_c
            };

            let player_response = PlayerResponse {
                tile_id :tile_id.to_string(),
                success : format!("tile updated with {}", tile.prop)
            };

            *tile_data = tile;

            let map_command = MapCommand {
                id : tile_data.id.clone(),
                info : MapCommandInfo::Touch()
            };

            let _ = context.tx_mc_webservice_realtime.send(map_command).await;


            let response = serde_json::to_vec(&player_response).unwrap();
            Ok(Response::new(Body::from(response)))
        },
        None => {

            let player_response = PlayerResponse {
                tile_id :tile_id.to_string(),
                success : "tile doesn't exist".to_owned()
            };
            let response = serde_json::to_vec(&player_response).unwrap();
            Ok(Response::new(Body::from(response)))
        }
    }
}

// TODO: Check for device_id already in use.

async fn handle_create_character(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, hyper::http::Error> {

    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: CharacterCreationRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);


    let generator = &context.working_game_map.id_generator;
    let new_id = generator.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    println!("got a {} as id base ",new_id);

    let stored_character = StoredCharacter{
        id: None,
        world_id: context.working_game_map.world_id.clone(),
        player_id: new_id,
        character_name: data.character_name.clone(),
        device_id: data.device_id.clone(),
        constitution: 50,
        health: 50
    };

    let data_collection: mongodb::Collection<StoredCharacter> = context.db_client.database("game").collection::<StoredCharacter>("players");
    let result = data_collection.insert_one(stored_character, None).await.unwrap();

    let object_id: Option<ObjectId> = match result.inserted_id {
        bson::Bson::ObjectId(id) => Some(id),
        _ => None,
    };

    let player_entity = PlayerEntity {
        object_id: object_id,
        character_name : data.character_name.clone(),
        // device_id: data.device_id,
        player_id: new_id,
        action: 0,
        position: [0.0, 0.0, 0.0],
        second_position: [0.0, 0.0, 0.0],
        constitution: 50,
        health: 50,
    };

    let mut players = context.working_game_map.players.lock().await;
    players.insert(new_id, player_entity.clone());
    drop(players);

    let mut players = context.storage_game_map.players.lock().await;
    players.insert(new_id, player_entity);

    drop(players);

    let new_character = CharacterCreationResponse{
        character_id: new_id,
        // name: "parker".to_owned(),
        // faction: "maya".to_owned(),
        // base_strenght: 1,
        // base_constitution: 1,
        // base_speed: 1,
        // base_intelligence: 1,
        // base_dexterity: 1,
    };

    let response = serde_json::to_vec(&new_character).unwrap();
    Ok(Response::new(Body::from(response)))
}

async fn handle_login_character(context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, hyper::http::Error> {

    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: JoinWithCharacterRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);

    let players = context.working_game_map.players.lock().await;


    if let Some(player) = players.get(&data.character_id) {

        let mut presentation_map = context.presentation_data.lock().await;


        let name_with_padding = format!("{: <5}", player.character_name);
        let name_data : Vec<u32> = name_with_padding.chars().into_iter().map(|c| c as u32).collect();
        let mut name_array = [0u32; 5];
        name_array.clone_from_slice(&name_data.as_slice()[0..5]);

        let player_presentation = PlayerPresentation {
            player_id: player.player_id,
            character_name: name_array,
        };

        presentation_map.insert(data.character_id, player_presentation.to_bytes());

        let saved_char = JoinWithCharacterResponse{
            character_id: player.player_id,
            tetrahedron_id:"j220132101".to_owned(),
            health: player.health,
            constitution: player.constitution,
        };

        let response = serde_json::to_vec(&saved_char).unwrap();
        Ok(Response::new(Body::from(response)))
    }
    else {
        Ok(Response::new(Body::from("error: char not found")))
    }

    
}

async fn handle_region_request(context: AppContext, data : Vec<&str>) -> Result<Response<Body>, hyper::http::Error> {

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

async fn handle_temp_region_request(context: AppContext, data : Vec<&str>) -> Result<Response<Body>, hyper::http::Error> {

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

async fn handle_players_request(context: AppContext) -> Result<Response<Body>, hyper::http::Error> {

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

async fn route(context: AppContext, req: Request<Body>) -> Result<Response<Body>, hyper::http::Error> {

    let uri = req.uri().to_string();
    let mut data = uri.split("/");
    data.next();
    if let Some(route) = data.next() {
        let rest : Vec<&str> = data.collect();
        match route {
            "region" => handle_region_request(context, rest).await,
            "temp_regions" => handle_temp_region_request(context, rest).await,
            "players_data" => handle_players_request(context).await,
            "character_creation" => handle_create_character(context, req).await,
            "join_with_character" => handle_login_character(context, req).await,
            "update_map_entity" => handle_update_map_entity(context, req).await,
            _ => {
                Ok(Response::new(Body::from("Resource not found")))
            }
        }
    }
    else {
            Ok(Response::new(Body::from("No resource defined")))
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
            println!("server saved the data, cleaning cache");

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