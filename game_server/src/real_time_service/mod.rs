pub mod client_handler;
pub mod utils;

use std::sync::Arc;
use std::{collections::HashMap};
use crate::map::map_entity::{MapEntity, MapCommand};
use crate::map::tetrahedron_id::TetrahedronId;
use crate::player::{player_action::PlayerAction, player_entity::PlayerEntity};
use crate::{client_state_system, web_service};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

pub fn start_server(
    tiles_lock: Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>,
    tile_command_tx: Sender<MapCommand>,
    tile_command_from_outside_rx : Receiver<MapCommand>,
    tile_changed_tx: Sender<MapEntity>,
) {

    let (server_state_tx, mut client_state_rx ) = tokio::sync::mpsc::channel::<Arc<Vec<[u8;508]>>>(200);
    let clients:HashMap<std::net::SocketAddr, PlayerEntity> = HashMap::new();
    let clients_mutex = std::sync::Arc::new(Mutex::new(clients));

    let server_lock = clients_mutex.clone();
    let server_send_to_clients_lock = clients_mutex.clone();

    let address: std::net::SocketAddr = "0.0.0.0:11004".parse().unwrap();
    // let address: std::net::SocketAddr = "127.0.0.1:11004".parse().unwrap();
    let udp_socket = Arc::new(utils::create_reusable_udp_socket(address));
    // let udp_socket = tokio::net::UdpSocket::bind(address).await.unwrap();

    let send_udp_socket = udp_socket.clone();
    // let read_udp_socket = udp_socket.clone();
    tokio::spawn(async move {
        loop {
            if let Some(packet_list) = client_state_rx.recv().await {
                let mut clients_data = server_send_to_clients_lock.lock().await;
                for client in clients_data.iter_mut()
                {
                    for packet in packet_list.iter(){
                        if client.1.player_id == 0 {
                            // let first_byte = packet[0]; // this is the protocol
                            let packet_sequence_number = u64::from_le_bytes(packet[1..9].try_into().unwrap());
                            println!("sending {}", packet_sequence_number);
                        }
                        let result = send_udp_socket.send_to(packet, client.0).await;
                        match result {
                            Ok(_) => {},
                            Err(_) => println!("error sending data through socket"),
                        }
                    }
                }
            }
        }
    });

    tokio::spawn(async move {
        let read_udp_socket = udp_socket.clone();
        let (from_client_to_world_tx, mut from_client_task_to_parent_rx ) = tokio::sync::mpsc::channel::<std::net::SocketAddr>(100);

        // each client has a client_action_tx where it can send updates to its own state
        // the consumer is the client state system, the system will summarize the requests and send them to each client.
        let (client_action_tx, client_action_rx ) = tokio::sync::mpsc::channel::<PlayerAction>(1000);


        // the first lock on clients data is used by the server to add and remove clients.

        // the second lock on clients_data is used for the client state system to send data to everyclient 
        // let process_lock = clients_mutex.clone();
        // this function will process all user actions and send to all players the global state
        // this looks inocent but will do a lot of work.
        // ---------------------------------------------------
        client_state_system::process_player_action(
            client_action_rx,
            tile_changed_tx,
            tile_command_from_outside_rx,
            tiles_lock,
            server_state_tx);
        // ---------------------------------------------------



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
                            // let (server_state_tx, client_state_rx ) = tokio::sync::mpsc::channel::<Arc<Vec<[u8;508]>>>(20);
                            let player_entity = PlayerEntity{
                                player_id : player_id, // we need to get this data from the packet
                                // tx : server_state_tx
                            };

                            clients_data.insert(from_address, player_entity);



                            // each client can send a message to remove itself using tx,
                            // each client can send actions to be processed using client_action_tx,
                            // each client can receive data to be sent to the client using client_state_rx because each client has its socket.
                            // the producer for this channel is saved in the player_entity which is saved on the clients_data
                            client_handler::spawn_client_process(player_id, address, from_address, tx, tile_command_tx.clone(), client_action_tx.clone(), buf_udp).await;
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


