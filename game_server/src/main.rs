use std::collections::HashSet;

use client_handler::ClientAction;

mod packet_router;
mod utils;
mod client_handler;
mod ping_protocol;
mod movement_protocol;
mod client_state_system;
mod player_state;




// #[tokio::main(worker_threads = 1)]
#[tokio::main]
async fn main() {
    
    let (from_client_to_world_tx, mut from_client_task_to_parent_rx ) = tokio::sync::mpsc::channel::<std::net::SocketAddr>(100);

    let initial_value = [0u8; 508];
    let (main_data_tx, childs_rx) = tokio::sync::watch::channel(initial_value);

    let (client_action_tx, client_action_rx ) = tokio::sync::mpsc::channel::<ClientAction>(1000);


    // this function will process all user actions and send to all players the global state
    // this looks inocent but will do a lot of work.
    // ---------------------------------------------------
    client_state_system::process_player_action(client_action_rx, main_data_tx);
    // ---------------------------------------------------


    let mut clients:HashSet<std::net::SocketAddr> = HashSet::new();

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
                    if !clients.contains(&from_address)
                    {
                        println!("--- create child!");
                        clients.insert(from_address);

                        let tx = from_client_to_world_tx.clone();
                        client_handler::spawn_client_process(address, from_address, tx, childs_rx.clone(), client_action_tx.clone(), buf_udp).await;
                    }
                }
            }
            Some(res) = from_client_task_to_parent_rx.recv() => {
                println!("removing entry from hash set");
                clients.remove(&res);
            }
            
        }
    }   
}


