
use std::{sync::Arc, collections::HashMap};

use serde::{Deserialize, Serialize};
use tokio::{sync::{Mutex, mpsc::{Receiver, Sender}}, time::error::Elapsed};
use warp::Filter;

use crate::{map::{tetrahedron_id::TetrahedronId, map_entity::MapEntity}, player};

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


async fn process_request(data : (PlayerRequest, Sender<MapEntity>, Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>)) -> Result<impl warp::Reply, warp::Rejection> {
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
            };

            let player_response = PlayerResponse {
                tile_id :tile_id.to_string(),
                success : format!("tile updated with {}", tile.prop)
            };

            *tile_data = tile;

            let _ = sender.send(tile_data.clone()).await;


            Ok(warp::reply::json(&player_response))
        },
        None => {
            let tile = MapEntity{
                id: tile_id.clone(),
                last_update: 13,
                health: 23,
                prop: 2,
            };
            tiles.insert(tile_id.clone(), tile.clone());
            let _ = sender.send(tile).await;

            let player_response = PlayerResponse {
                tile_id :tile_id.to_string(),
                success : "new tile added".to_owned()
            };
            Ok(warp::reply::json(&player_response))
        }
    }
}

pub fn start_server(tiles_lock: Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>, tile_changed_rx : Sender<MapEntity>) {
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
            .map(move |player_request|{
                (player_request, rx.clone(), lock.clone())
            })
            .and_then(process_request);
    
            warp::serve(promote).run(([127, 0, 0, 1], 3030)).await
        }
    });
}