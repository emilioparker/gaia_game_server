use std::{collections::HashMap, sync::mpsc::Sender};
use game_server::{player_action::ClientAction, client_state_system, utils, client_handler, player_state::PlayerState};
use tokio::sync::Mutex;

// #[tokio::main(worker_threads = 1)]
#[tokio::main]
async fn main() {
    
    let (from_client_to_world_tx, mut from_client_task_to_parent_rx ) = tokio::sync::mpsc::channel::<std::net::SocketAddr>(100);


    let (client_action_tx, client_action_rx ) = tokio::sync::mpsc::channel::<ClientAction>(1000);


    let mut clients:HashMap<std::net::SocketAddr, tokio::sync::mpsc::Sender<Vec<PlayerState>>> = HashMap::new();
    let mut clients_mutex = std::sync::Arc::new(Mutex::new(clients));
    let server_lock = clients_mutex.clone();
    let process_lock = clients_mutex.clone();
    // this function will process all user actions and send to all players the global state
    // this looks inocent but will do a lot of work.
    // ---------------------------------------------------
    client_state_system::process_player_action(client_action_rx, process_lock);
    // ---------------------------------------------------

    let address: std::net::SocketAddr = "0.0.0.0:11004".parse().unwrap();
    // let address: std::net::SocketAddr = "127.0.0.1:11004".parse().unwrap();
    let udp_socket = utils::create_reusable_udp_socket(address);

    let mut buf_udp = [0u8; 508];
    loop {
        let socket_receive = udp_socket.recv_from(&mut buf_udp);

        tokio::select! {
            result = socket_receive => {
                if let Ok((size, from_address)) = result {
                    println!("Parent: {:?} bytes received from {}", size, from_address);
                    let mut clients_data = server_lock.lock().await;
                    if !clients_data.contains_key(&from_address)
                    {
                        println!("--- create child!");
                        let tx = from_client_to_world_tx.clone();
                        let (server_state_tx, client_state_rx ) = tokio::sync::mpsc::channel::<Vec<PlayerState>>(20);
                        clients_data.insert(from_address, server_state_tx);
                        client_handler::spawn_client_process(address, from_address, tx, client_state_rx, client_action_tx.clone(), buf_udp).await;
                    }
                }
            }
            Some(res) = from_client_task_to_parent_rx.recv() => {
                println!("removing entry from hash set");
                let mut clients_data = server_lock.lock().await;
                clients_data.remove(&res);
            }
        }
    }   
}


