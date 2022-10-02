
use std::{sync::Arc, collections::HashMap};

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, mpsc::{Receiver, Sender}};
use warp::Filter;

use crate::map::{tetrahedron_id::TetrahedronId, map_entity::MapEntity};

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

pub fn start_server(tiles_lock: Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>, tile_changed_rx : Sender<TetrahedronId>) {
    tokio::spawn(async move {

        'receive_loop : loop {
            let promote = warp::post()
            .and(warp::path("process_request"))
            // .and(warp::path::param::<u32>())
            // Only accept bodies smaller than 16kb...
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json())
            .map(|mut task: PlayerRequest| {
                println!("{:?}", task);
                let mut tile_id = TetrahedronId::from_string(&task.tile_id);
                tile_id.area = 19;
                // here we should set the data and indicate that a tile changed so other players can see the change
                let player_response = PlayerResponse {
                    tile_id :tile_id.to_string(),
                    success : "true".to_owned()
                };

                warp::reply::json(&player_response)
            });
    
        warp::serve(promote).run(([127, 0, 0, 1], 3030)).await
        }
    });
}