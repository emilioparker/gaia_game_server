pub mod client_handler;
pub mod utils;

use std::sync::Arc;
use std::{collections::HashMap};
use crate::map::GameMap;
use crate::map::map_entity::{MapEntity, MapCommand};
use crate::player::player_connection::PlayerConnection;
use crate::player::{player_action::PlayerAction, player_entity::PlayerEntity};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

pub fn start_server() -> (Receiver<MapCommand>, Receiver<PlayerAction>, Sender<Arc<Vec<Vec<u8>>>>) {

    let (tx_mc_client_statesys, rx_mc_client_statesys) = tokio::sync::mpsc::channel::<MapCommand>(200);
    let (tx_bytes_statesys_socket, mut rx_bytes_state_socket ) = tokio::sync::mpsc::channel::<Arc<Vec<Vec<u8>>>>(200);
    let (tx_pa_client_statesys, rx_pa_client_statesys) = tokio::sync::mpsc::channel::<PlayerAction>(1000);

    let client_connections:HashMap<std::net::SocketAddr, PlayerConnection> = HashMap::new();
    let client_connections_mutex = std::sync::Arc::new(Mutex::new(client_connections));

    let server_lock = client_connections_mutex.clone();
    let server_send_to_clients_lock = client_connections_mutex.clone();

    let address: std::net::SocketAddr = "0.0.0.0:11004".parse().unwrap();
    let udp_socket = Arc::new(utils::create_reusable_udp_socket(address));
    let send_udp_socket = udp_socket.clone();

    tokio::spawn(async move {
        loop {
            if let Some(packet_list) = rx_bytes_state_socket.recv().await {
                let mut clients_data = server_send_to_clients_lock.lock().await;
                for client in clients_data.iter_mut()
                {
                    for packet in packet_list.iter(){
                        if client.1.player_id == 0 {
                            // let first_byte = packet[0]; // this is the protocol
                            // the packet is compress, I can't read the sequence number
                            // let packet_sequence_number = u64::from_le_bytes(packet[1..9].try_into().unwrap());
                            // println!("sending {}", packet_sequence_number);
                        }
                        // todo: only send data if client is correctly validated, add state to clients_data
                        println!("sending packet to clients ");
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

        //use to communicate that the client disconnected
        let (tx_addr_client_realtime, mut rx_addr_client_realtime ) = tokio::sync::mpsc::channel::<std::net::SocketAddr>(100);

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

                            println!("--- create child for {}", player_id);
                            // we need to create a struct that contains the tx and some client data that we can use to filter what we
                            // send, this will be epic
                            // let (server_state_tx, client_state_rx ) = tokio::sync::mpsc::channel::<Arc<Vec<[u8;508]>>>(20);
                            let player_entity = PlayerConnection{
                                player_id : player_id, // we need to get this data from the packet
                                // tx : server_state_tx
                            };

                            clients_data.insert(from_address, player_entity);



                            // each client can send a message to remove itself using tx,
                            // each client can send actions to be processed using client_action_tx,
                            // each client can receive data to be sent to the client using client_state_rx because each client has its socket.
                            // the producer for this channel is saved in the player_entity which is saved on the clients_data
                            client_handler::spawn_client_process(
                                player_id, 
                                address, 
                                from_address, 
                                tx_addr_client_realtime.clone(), 
                                tx_mc_client_statesys.clone(), 
                                tx_pa_client_statesys.clone(), 
                                buf_udp,
                            ).await;
                        }
                        else
                        {
                            println!("rejected");
                        }
                    }
                }
                Some(res) = rx_addr_client_realtime.recv() => {
                    println!("removing entry from hash set");
                    let mut clients_data = server_lock.lock().await;
                    clients_data.remove(&res);
                }
            }
        }   
    });

    (rx_mc_client_statesys, rx_pa_client_statesys, tx_bytes_statesys_socket)
}


