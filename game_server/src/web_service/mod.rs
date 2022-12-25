
use std::{sync::Arc};

use hyper::{Request, body, server::conn::AddrStream};
use hyper_static::serve::static_file;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Response, Server};
use hyper::service::{make_service_fn, service_fn};

use crate::map::GameMap;
use crate::map::map_entity::{MapCommand, MapEntity, MapCommandInfo};
use crate::map::tetrahedron_id::TetrahedronId;

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

#[derive(Clone)]
struct AppContext {
    game_map : Arc<GameMap>,
    map_command_sender : Sender<MapCommand>
}

async fn handle(context: AppContext, mut req: Request<Body>) -> Result<Response<Body>, Infallible> {

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

            let _ = context.map_command_sender.send(map_command).await;


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

async fn handle_file_request(req: Request<Body>) -> Result<Response<Body>, hyper::http::Error> {

    let uri = req.uri().to_string();
    let mut data = uri.split("/region/");
    let _ = data.next();
    let region = data.next();
    if let Some(region) = region {
        println!("this is the region {}", region);
        // let file_path = format!("map_initial_data/world_002_{}_props.bytes", region);
        let file_path = format!("map_working_data/world_002_{}_props.bytes", region);
        let path = std::path::Path::new(&file_path);
        return match static_file(
            &path,
            // Some("application/octet-stream"), // mime type
            Some("text/html"), // mime type
            &req.headers(), // hyper request header map
            65536 // buffer size
        )
        .await
        {
            Ok(v) => v, // return it
            Err(e) => e.into(), // transform the error and return
        };
    }
    else{
        println!("bad request");
        return Ok(Response::new(Body::from("bad request")));
    }

}

pub fn start_server(map: Arc<GameMap>, tile_changed_rx : Sender<MapCommand>) {

    let context = AppContext {
        game_map : map,
        map_command_sender : tile_changed_rx
    };

    tokio::spawn(async move {
        let addr = SocketAddr::from(([0, 0, 0, 0], 3030));
        let make_service = make_service_fn(move |conn: &AddrStream| {
            let context = context.clone();
            let _addr = conn.remote_addr();
            let service = service_fn(move |req| {
                handle(context.clone(), req)
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

    // let context = AppContext {
    //     game_map : map,
    // };

    tokio::spawn(async move {

        let make_service = make_service_fn(|_| {
            // future::ok::<_, hyper::Error>(service_fn(move |req| handle_request(req, static_.clone())))
            let service = service_fn(move |req| {
                handle_file_request(req)
            });
            async move { Ok::<_, Infallible>(service) }
        });

        let addr = ([0, 0, 0, 0], 3031).into();
        let server = hyper::Server::bind(&addr).serve(make_service);
        eprintln!("Doc server running on http://{}/", addr);
        server.await.expect("Server failed");
    });
}