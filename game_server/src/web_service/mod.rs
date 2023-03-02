
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64};
use std::io::Write;
use std::{sync::Arc};

use bson::oid::ObjectId;
use futures_util::future::OrElse;
use futures_util::lock::Mutex;
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
use crate::map::GameMap;
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
    db_client : mongodb ::Client
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
            tetrahedron_id:"d130001211".to_owned(),
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
    let region = iterator.next();
    if let Some(region) = region {

        let data_collection: mongodb::Collection<StoredRegion> = context.db_client.database("game").collection::<StoredRegion>("regions");

        // Look up one document:
        let data_from_db: Option<StoredRegion> = data_collection
        .find_one(
            bson::doc! {
                    "world_id": context.storage_game_map.world_id,
                    "region_id": region.to_owned()
            },
            None,
        ).await
        .unwrap();

        if let Some(region_from_db) = data_from_db {

            let binary_data: Vec<u8> = match region_from_db.compressed_data {
                bson::Bson::Binary(binary) => binary.bytes,
                _ => panic!("Expected Bson::Binary"),
            };

            let response = Response::builder()
                .status(hyper::StatusCode::OK)
                .header("Content-Type", "application/octet-stream")
                .body(Body::from(binary_data))
                .expect("Failed to create response");
            Ok(response)
        }
        else {
            Ok(Response::new(Body::from("Error getting region from db")))
        }
    }
    else{
        println!("bad request");
        return Ok(Response::new(Body::from("bad request")));
    }

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
            encoder.write(bytes).unwrap();
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

pub fn start_server(
    working_map: Arc<GameMap>, 
    storage_map: Arc<GameMap>, 
    db_client : mongodb :: Client) 
    -> Receiver<MapCommand>{

    let (tx_mc_webservice_gameplay, rx_mc_webservice_gameplay ) = tokio::sync::mpsc::channel::<MapCommand>(200);
    
    let context = AppContext {
        presentation_data : Arc::new(Mutex::new(HashMap::new())),
        working_game_map : working_map,
        storage_game_map : storage_map,
        tx_mc_webservice_realtime : tx_mc_webservice_gameplay,
        db_client : db_client,
        last_presentation_update: Arc::new(AtomicU64::new(0)),
        compressed_presentation_data: Arc::new(Mutex::new(Vec::new())),
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

    rx_mc_webservice_gameplay
}