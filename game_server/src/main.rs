use std::collections::HashMap;
use std::sync::Arc;

use game_server::map::map_entity::MapCommand;
use game_server::map::map_entity::MapEntity;
use game_server::map::tetrahedron_id::TetrahedronId;
use tokio::sync::Mutex;
use game_server::real_time_service;
use game_server::web_service;

// #[tokio::main(worker_threads = 1)]
#[tokio::main]
async fn main() {

    //console_subscriber::init();
    // tiles are modified by many systems, but since we only have one core... our mutex doesn't work too much
    let all_tiles = HashMap::<TetrahedronId,MapEntity>::new();
    let tiles_mutex = Arc::new(Mutex::new(all_tiles));
    let realtime_tiles_service_lock = tiles_mutex.clone();
    let webservice_tiles_lock = tiles_mutex.clone();

    let (map_command_tx, real_time_service_rx ) = tokio::sync::mpsc::channel::<MapCommand>(20);
    let web_service_map_commands_tx = map_command_tx.clone();
    let client_map_commands_tx = map_command_tx.clone();
    real_time_service::start_server(realtime_tiles_service_lock, client_map_commands_tx, real_time_service_rx);

    web_service::start_server(webservice_tiles_lock, web_service_map_commands_tx);

    loop{
        tokio::task::yield_now().await;
    }
}
