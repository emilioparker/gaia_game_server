use std::{collections::HashMap};
use game_server::{player_action::PlayerAction, client_state_system, utils, client_handler, player_state::PlayerState, player_entity::PlayerEntity};
use tokio::sync::Mutex;

// #[tokio::main(worker_threads = 1)]
#[tokio::main]
async fn main() {
    
    let (from_client_to_world_tx, mut from_client_task_to_parent_rx ) = tokio::sync::mpsc::channel::<std::net::SocketAddr>(100);
    let (client_action_tx, client_action_rx ) = tokio::sync::mpsc::channel::<PlayerAction>(1000);

    let clients:HashMap<std::net::SocketAddr, PlayerEntity> = HashMap::new();
    let clients_mutex = std::sync::Arc::new(Mutex::new(clients));

    let server_lock = clients_mutex.clone();
    let process_lock = clients_mutex.clone();
    // this function will process all user actions and send to all players the global state
    // this looks inocent but will do a lot of work.
    // ---------------------------------------------------
    client_state_system::process_player_action(client_action_rx,  process_lock);
    // ---------------------------------------------------

    let address: std::net::SocketAddr = "0.0.0.0:11004".parse().unwrap();
    // let address: std::net::SocketAddr = "127.0.0.1:11004".parse().unwrap();
    let udp_socket = utils::create_reusable_udp_socket(address);

    let mut buf_udp = [0u8; 508];
    loop {
        let socket_receive = udp_socket.recv_from(&mut buf_udp);


        // tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;

        tokio::select! {
            result = socket_receive => {
                if let Ok((size, from_address)) = result {
                    println!("Parent: {:?} bytes received from {}", size, from_address);
                    let mut clients_data = server_lock.lock().await;
                    if !clients_data.contains_key(&from_address)
                    {
                        // byte 0 is for the protocol, and we are sure the next 8 bytes are for the id.
                        let start = 1;
                        let end = start + 8;
                        let player_id = u64::from_le_bytes(buf_udp[start..end].try_into().unwrap());
                        // start = end;
                        // end = start + 8;

                        println!("--- create child for {}", player_id);
                        let tx = from_client_to_world_tx.clone();
                        // we need to create a struct that contains the tx and some client data that we can use to filter what we
                        // send, this will be epic
                        let (server_state_tx, client_state_rx ) = tokio::sync::mpsc::channel::<Vec<PlayerState>>(20);
                        let player_entity = PlayerEntity{
                            sequence_number : 0,
                            player_id : player_id, // we need to get this data from the packet
                            tx : server_state_tx
                        };

                        clients_data.insert(from_address, player_entity);
                        client_handler::spawn_client_process(address, from_address, tx, client_state_rx, client_action_tx.clone(), buf_udp).await;
                    }
                    else
                    {
                        println!("rejected");
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


