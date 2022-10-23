
use std::{sync::Arc, collections::HashMap};

use serde::{Deserialize, Serialize};
use tokio::{sync::{Mutex, mpsc::{Receiver, Sender}}, time::error::Elapsed};
use warp::Filter;

use crate::{map::{tetrahedron_id::TetrahedronId, map_entity::{MapEntity, MapCommand, MapCommandInfo}}, player};

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


async fn process_request(data : (PlayerRequest, Sender<MapCommand>, Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>)) -> Result<impl warp::Reply, warp::Rejection> {

    let tile_id = TetrahedronId::from_string(&data.0.tile_id);
    // tile_id.area = 19;
    // here we should set the data and indicate that a tile changed so other players can see the change

    let mut tiles = data.2.lock().await;

    let sender = data.1;
    let tile_data = tiles.get_mut(&tile_id);

    match tile_data {
        Some(tile_data) => {

            let tile = MapEntity{
                id: tile_data.id.clone(),
                last_update: tile_data.last_update,
                health: tile_data.health,
                prop: data.0.prop,
                heights: [0,1,2],
                normal_a: [1.2,1.1,1.5],
                normal_b: [1.2,1.1,1.6],
                normal_c: [1.2,1.1,1.7],
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

            let _ = sender.send(map_command).await;


            Ok(warp::reply::json(&player_response))
        },
        None => {
            let tile = MapEntity{
                id: tile_id.clone(),
                last_update: 13,
                health: 23,
                prop: data.0.prop,
                heights: [0,1,2],
                normal_a: [1.2,1.1,1.5],
                normal_b: [1.2,1.1,1.6],
                normal_c: [1.2,1.1,1.7],
            };
            tiles.insert(tile_id.clone(), tile.clone());

            let map_command = MapCommand {
                id : tile.id.clone(),
                info : MapCommandInfo::Touch()
            };

            let _ = sender.send(map_command).await;

            let player_response = PlayerResponse {
                tile_id :tile_id.to_string(),
                success : "new tile added".to_owned()
            };
            Ok(warp::reply::json(&player_response))
        }
    }
}


pub fn start_server(tiles_lock: Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>, tile_changed_rx : Sender<MapCommand>) {
    tokio::spawn(async move {

        'receive_loop : loop {
            let rx = tile_changed_rx.clone();
            let lock = tiles_lock.clone();
            let promote = warp::post()
            .and(warp::path("process_request"))
            // .and(warp::path::param::<u32>())
            // Only accept bodies smaller than 16kb...
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json())
            
            .map(move |player_request : PlayerRequest|{
                // warp::reply::json(&2u32)
                (player_request, rx.clone(), lock.clone())
            })
            .and_then(process_request);
    
            warp::serve(promote).run(([0, 0, 0, 0], 3030)).await
        }
    });
}