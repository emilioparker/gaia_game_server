use std::collections::HashMap;
use std::sync::Arc;

use game_server::map::map_entity::MapEntity;
use game_server::map::tetrahedron_id::TetrahedronId;
use tokio::sync::Mutex;
use game_server::real_time_service;
use game_server::web_service;

// #[tokio::main(worker_threads = 1)]
#[tokio::main]
async fn main() {

    // tiles are modified by many systems, but since we only have one core... our mutex doesn't work too much
    let all_tiles = HashMap::<TetrahedronId,MapEntity>::new();
    let tiles_mutex = Arc::new(Mutex::new(all_tiles));
    let tiles_processor_lock = tiles_mutex.clone();
    let tiles_agregator_lock = tiles_mutex.clone();

    real_time_service::start_server();
    web_service::start_server();

    loop{
        tokio::task::yield_now().await;
    }
}
