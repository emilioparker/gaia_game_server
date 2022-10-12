pub mod client_handler;
pub mod utils;

use std::sync::Arc;
use std::{collections::HashMap};
use crate::map::map_entity::{MapEntity, MapCommand};
use crate::map::tetrahedron_id::TetrahedronId;
use crate::player::{player_action::PlayerAction, player_state::PlayerState, player_entity::PlayerEntity};
use crate::real_time_service::client_handler::StateUpdate;
use crate::{client_state_system, web_service};
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

pub fn start_server(tiles_lock: Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>,
    tile_changed_tx: Sender<MapCommand>,
    tile_changed_rx : Receiver<MapCommand>) {
    tokio::spawn(async move {
        let (from_client_to_world_tx, mut from_client_task_to_parent_rx ) = tokio::sync::mpsc::channel::<std::net::SocketAddr>(100);

        // each client has a client_action_tx where it can send updates to its own state
        // the consumer is the client state system, the system will summarize the requests and send them to each client.
        let (client_action_tx, client_action_rx ) = tokio::sync::mpsc::channel::<PlayerAction>(1000);

        let clients:HashMap<std::net::SocketAddr, PlayerEntity> = HashMap::new();
        let clients_mutex = std::sync::Arc::new(Mutex::new(clients));

        // the first lock on clients data is used by the server to add and remove clients.
        let server_lock = clients_mutex.clone();

        // the second lock on clients_data is used for the client state system to send data to everyclient 
        let process_lock = clients_mutex.clone();
        // this function will process all user actions and send to all players the global state
        // this looks inocent but will do a lot of work.
        // ---------------------------------------------------
        client_state_system::process_player_action(client_action_rx, tile_changed_rx, tiles_lock, process_lock);
        // ---------------------------------------------------

        let listener = tokio::net::TcpListener::bind("0.0.0.0:11004").await.unwrap();
        // let address: std::net::SocketAddr = "0.0.0.0:11004".parse().unwrap();
        // let address: std::net::SocketAddr = "127.0.0.1:11004".parse().unwrap();


        // let udp_socket = utils::create_reusable_udp_socket(address);
        // let (tokio_tcp_stream, _) = udp_socket.accept().await?;

        let mut buf_udp = [0u8; 508];
        loop {
            // let (socket, _) = listener.accept().await?;
            // process_socket(socket).await;
            // let socket_receive = udp_socket.read(&mut buf_udp);


            // tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;

            tokio::select! {
                result = listener.accept() => {
                    if let Ok((stream, from_address)) = result {
                        // println!("Parent: {:?} bytes received from {}", size, from_address);
                        let mut clients_data = server_lock.lock().await;
                        if !clients_data.contains_key(&from_address)
                        {
                            // byte 0 is for the protocol, and we are sure the next 8 bytes are for the id.
                            let start = 1;
                            let end = start + 8;
                            // let player_id = u64::from_le_bytes(buf_udp[start..end].try_into().unwrap());
                            // start = end;
                            // end = start + 8;

                            println!("--- create child for {}", from_address);
                            let tx = from_client_to_world_tx.clone();
                            // we need to create a struct that contains the tx and some client data that we can use to filter what we
                            // send, this will be epic
                            let (server_state_tx, client_state_rx ) = tokio::sync::mpsc::channel::<Arc<Vec<[u8;508]>>>(20);
                            let player_entity = PlayerEntity{
                                player_id : 0, // we need to get this data from the packet
                                tx : server_state_tx
                            };

                            clients_data.insert(from_address, player_entity);



                            // each client can send a message to remove itself using tx,
                            // each client can send actions to be processed using client_action_tx,
                            // each client can receive data to be sent to the client using client_state_rx because each client has its socket.
                            // the producer for this channel is saved in the player_entity which is saved on the clients_data
                            client_handler::spawn_client_process(0, stream, from_address, tx, client_state_rx, tile_changed_tx.clone(), client_action_tx.clone(), buf_udp).await;
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
    });
}


