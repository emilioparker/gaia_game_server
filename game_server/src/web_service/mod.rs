
use std::collections::HashMap;
use std::{sync::Arc};

use hyper::{Request, body, server::conn::AddrStream};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;

use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Response, Server};
use hyper::service::{make_service_fn, service_fn};

use crate::long_term_storage_service::db_region::StoredRegion;
use crate::map::GameMap;
use crate::map::map_entity::{MapCommand, MapEntity, MapCommandInfo};
use crate::map::tetrahedron_id::TetrahedronId;
use crate::player::player_entity::PlayerEntity;

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

    name: String, //create
    faction: String, // tree
}

#[derive(Deserialize, Serialize, Debug)]
struct CharacterCreationResponse {
    character_id:u32,
    name: String, 
    faction: String,
    base_strenght: u32,
    base_constitution: u32,
    base_speed: u32,
    base_intelligence: u32,
    base_dexterity: u32
}


#[derive(Clone)]
struct AppContext {
    players : Arc<Mutex<HashMap<u64, PlayerEntity>>>,
    game_map : Arc<GameMap>,
    tx_mc_webservice_realtime : Sender<MapCommand>,
    db_client : mongodb ::Client
}

async fn handle_update_map_entity(context: AppContext, mut req: Request<Body>) -> Result<Response<Body>, hyper::http::Error> {

    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: PlayerRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);
    let tile_id = TetrahedronId::from_string(&data.tile_id);
    let region = context.game_map.get_region_from_child(&tile_id);
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
                heat : tile_data.heat,
                moisture : tile_data.moisture,
                biome : tile_data.biome,
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

async fn handle_create_character(_context: AppContext, mut req: Request<Body>) ->Result<Response<Body>, hyper::http::Error> {

    let body = req.body_mut();
    let data = body::to_bytes(body).await.unwrap();
    let data: CharacterCreationRequest = serde_json::from_slice(&data).unwrap();
    println!("handling request {:?}", data);

    let new_character = CharacterCreationResponse{
        character_id: 1,
        name: "parker".to_owned(),
        faction: "maya".to_owned(),
        base_strenght: 1,
        base_constitution: 1,
        base_speed: 1,
        base_intelligence: 1,
        base_dexterity: 1,
    };

    let response = serde_json::to_vec(&new_character).unwrap();
    Ok(Response::new(Body::from(response)))
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

async fn route(context: AppContext, req: Request<Body>) -> Result<Response<Body>, hyper::http::Error> {

    let uri = req.uri().to_string();
    let mut data = uri.split("/");
    data.next();
    if let Some(route) = data.next() {
        let rest : Vec<&str> = data.collect();
        match route {
            "region" => handle_region_request(context, rest).await,
            "character_creation" => handle_create_character(context, req).await,
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
    players: Arc<Mutex<HashMap<u64, PlayerEntity>>>, 
    map: Arc<GameMap>, 
    tx_mc_webservice_realtime : Sender<MapCommand>,
    db_client : mongodb :: Client) {

    let context = AppContext {
        players: players,
        game_map : map,
        tx_mc_webservice_realtime : tx_mc_webservice_realtime,
        db_client : db_client
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
}