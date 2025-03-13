pub mod client_handler;
pub mod utils;

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::collections::HashMap;
use crate::gameplay_service::generic_command::GenericCommand;
use crate::mob::mob_command::MobCommand;
use crate::ServerState;
use crate::chat::ChatCommand;
use crate::map::GameMap;
use crate::map::map_entity::MapCommand;
use crate::character::character_command::CharacterCommand;
use crate::tower::TowerCommand;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

pub enum DataType
{
    NoData = 25,
    PlayerState = 26,
    TileState = 27,
    PlayerPresentation = 28,
    Attack = 29,
    PlayerReward = 30,
    TowerState = 32,
    ChatMessage = 33,
    ServerStatus = 34,
    MobStatus = 35,
    AttackDetails = 36,
}

pub fn start_server(
    map : Arc<GameMap>,
    server_state: Arc<ServerState>
) -> (
    Receiver<MapCommand>,
    Receiver<MobCommand>,
    Receiver<CharacterCommand>, 
    Receiver<TowerCommand>, 
    Receiver<ChatCommand>,
    Sender<Vec<(u64,u8,Vec<u8>)>>
) // packet number, faction, data
{
    let (tx_bytes_client_socket, mut rx_bytes_client_socket) = tokio::sync::mpsc::channel::<GenericCommand>(1000);
    let (tx_mc_client_statesys, rx_mc_client_statesys) = tokio::sync::mpsc::channel::<MapCommand>(1000);
    let (tx_moc_client_statesys, rx_moc_client_statesys) = tokio::sync::mpsc::channel::<MobCommand>(1000);
    let (tx_bytes_statesys_socket, mut rx_bytes_state_socket ) = tokio::sync::mpsc::channel::<Vec<(u64, u8, Vec<u8>)>>(1000);
    let (tx_pc_client_statesys, rx_pc_client_statesys) = tokio::sync::mpsc::channel::<CharacterCommand>(1000);
    let (tx_tc_client_statesys, rx_tc_client_statesys) = tokio::sync::mpsc::channel::<TowerCommand>(1000);
    let (tx_cc_client_statesys, rx_cc_client_statesys) = tokio::sync::mpsc::channel::<ChatCommand>(1000);

    server_state.tx_mc_client_gameplay.store(tx_mc_client_statesys.capacity() as f32 as u16, std::sync::atomic::Ordering::Relaxed);
    server_state.tx_pc_client_gameplay.store(tx_pc_client_statesys.capacity() as f32 as u16, std::sync::atomic::Ordering::Relaxed);
    server_state.tx_tc_client_gameplay.store(tx_tc_client_statesys.capacity() as f32 as u16, std::sync::atomic::Ordering::Relaxed);
    server_state.tx_cc_client_gameplay.store(tx_cc_client_statesys.capacity() as f32 as u16, std::sync::atomic::Ordering::Relaxed);

    let packet_builder_server_state = server_state.clone();
    let client_connections:HashMap<std::net::SocketAddr, (u16, u8)> = HashMap::new();
    let client_connections_mutex = std::sync::Arc::new(Mutex::new(client_connections));

    let server_lock = client_connections_mutex.clone();
    let server_send_to_clients_lock = client_connections_mutex.clone();

    let address: std::net::SocketAddr = "0.0.0.0:11004".parse().unwrap();
    let udp_socket = Arc::new(utils::create_reusable_udp_socket(address));
    let send_udp_socket = udp_socket.clone();
    let send_directly_udp_socket = udp_socket.clone();

    let mut previous_packages : VecDeque<(u64, u8, Vec<u8>)> = VecDeque::new();

    // let mut missing_packages_record = []
    let mut player_missing_packets = HashMap::<u16, [AtomicU64;10]>::new();

    let mut i:u16 = 0;

    while i < u16::MAX {
        i = i + 1;
        let data = [
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
        ];
        player_missing_packets.insert(i, data);
    }

    let shared_player_missing_packets = Arc::new(player_missing_packets);
    let _executer_shared_player_missing_packets = shared_player_missing_packets.clone();
    let updater_shared_player_missing_packets = shared_player_missing_packets.clone();

    tokio::spawn(async move 
    {
        loop 
        {
            if let Some(command) = rx_bytes_client_socket.recv().await 
            {
                let result = send_directly_udp_socket.try_send_to(&command.data, command.player_address);
                match result 
                {
                    Ok(_) => 
                    {
                        // cli_log::info!("ping sent to client ");
                    },
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock =>
                    {
                        cli_log::info!("error sending specific data to {} would block", command.player_address);
                    },
                    Err(_) => cli_log::info!("error sending specific data through socket"),
                }
            }
        }
    });

    tokio::spawn(async move 
    {
        loop 
        {
            // there are two sources of packets, chat and game. Each one has a differente packet id.
            if let Some(packet_list) = rx_bytes_state_socket.recv().await 
            {
                let mut first_client = true;
                let mut sent_bytes : u64 = 0;
                let mut clients_data = server_send_to_clients_lock.lock().await;
                for client in clients_data.iter_mut()
                {
                    for (_packet_id, faction, data) in packet_list.iter()
                    {
                        sent_bytes += data.len() as u64;
                        // cli_log::info!("sending packet with id {packet_id}");
                        if client.1.0 == 0u16 
                        {
                            // let first_byte = packet[0]; // this is the protocol
                            // the packet is compress, I can't read the sequence number
                            // let packet_sequence_number = u64::from_le_bytes(packet[1..9].try_into().unwrap());
                            // cli_log::info!("sending {}", packet_sequence_number);
                        }
                        // todo: only send data if client is correctly validated, add state to clients_data
                        if client.1.1 == *faction || *faction == 0
                        {
                            let result = send_udp_socket.try_send_to(data, client.0.clone());
                            match result 
                            {
                                Ok(_) => 
                                {
                                    // cli_log::info!("data sent to client {}", packet.len());
                                },
                                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock =>
                                {
                                    cli_log::info!("error sending data would block");
                                },
                                Err(_) => cli_log::info!("error sending data through socket"),
                            }
                        }

                        // we will try to send missing packages.

                        // cli_log::info!("sending missing packets for {}", client.1.0);
                        // if let Some(missing_packages_for_player) = executer_shared_player_missing_packets.get(&client.1.0) 
                        // {
                        //     // this should never fail
                        //     for missing_packet in missing_packages_for_player
                        //     {
                        //         let packet_id = missing_packet.load(std::sync::atomic::Ordering::Relaxed);
                        //         if packet_id != 0 
                        //         {
                        //             if let Some((old_id, _faction, old_data)) = previous_packages.iter().find(|(id, _faction, _data)| packet_id == *id)
                        //             {
                        //                 // sending missing data if found
                        //                 cli_log::info!("sending missing packet with id {packet_id}");
                        //                 let result = send_udp_socket.try_send_to(old_data, client.0.clone());
                        //                 match result {
                        //                     Ok(_) => {
                        //                         // cli_log::info!("data sent to client {}", packet.len());
                        //                     },
                        //                     Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock =>
                        //                     {
                        //                         cli_log::info!("error sending old data would block {}", old_id);
                        //                     },
                        //                     Err(_) => cli_log::info!("error sending old data through socket {} ", old_id),
                        //                 }
                        //             }
                        //         }
                        //     }
                        // }
                    }

                    if first_client
                    {
                        first_client = false;
                        packet_builder_server_state.sent_bytes.fetch_add(sent_bytes, std::sync::atomic::Ordering::Relaxed);
                    }
                }

                for packet in packet_list.into_iter()
                {
                    // only global packets are stored. global is 0 in the faction field
                    if packet.1 == 0 
                    {
                        previous_packages.push_front(packet);
                        if previous_packages.len() > 100
                        {
                            let _pop_result = previous_packages.pop_back();
                        }
                    }
                }
                // cli_log::info!("storing packages {}", previous_packages.len());
            }
        }
    });

    tokio::spawn(async move {

        // let client_send_udp_socket = clients_send_udp_socket.clone();
        //use to communicate that the client disconnected
        let (tx_addr_client_realtime, mut rx_addr_client_realtime ) = tokio::sync::mpsc::channel::<(std::net::SocketAddr, u64)>(100);

        let mut buf_udp = [0u8; 508];
        loop {
            let socket_receive = udp_socket.recv_from(&mut buf_udp);

            // tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;

            tokio::select! 
            {
                result = socket_receive => 
                {
                    if let Ok((packet_size, from_address)) = result 
                    {
                        cli_log::info!("Parent: {:?} bytes received from {}", packet_size, from_address);
                        let mut clients_data = server_lock.lock().await;
                        if !clients_data.contains_key(&from_address)
                        {
                            // byte 0 is for the protocol, and we are sure the next 8 bytes are for the id.
                            let start = 1;
                            let end = start + 8;
                            let player_session_id = u64::from_le_bytes(buf_udp[start..end].try_into().unwrap());

                            let start = end;
                            let end = start + 2;
                            let player_id = u16::from_le_bytes(buf_udp[start..end].try_into().unwrap());

                            let start = end;
                            // let end = start + 1;
                            let faction = buf_udp[start];

                            cli_log::info!("--- create child for {} with session id {}", player_id, player_session_id);
                            let stored_session_id = &map.logged_in_players[player_id as usize];
                            let session_id = stored_session_id.load(std::sync::atomic::Ordering::Relaxed);
                            cli_log::info!("comparing {} with server {}", player_session_id, session_id);

                            if session_id == player_session_id  && session_id != 0
                            {
                                clients_data.insert(from_address, (player_id, faction));

                                server_state.online_players.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                // each client can send a message to remove itself using tx,
                                // each client can send actions to be processed using client_action_tx,
                                // each client can receive data to be sent to the client using client_state_rx because each client has its socket.
                                // the producer for this channel is saved in the player_entity which is saved on the clients_data
                                client_handler::spawn_client_process(
                                    player_id, 
                                    session_id,
                                    address, 
                                    from_address, 
                                    map.clone(),
                                    server_state.clone(),
                                    tx_bytes_client_socket.clone(),
                                    tx_addr_client_realtime.clone(), 
                                    tx_mc_client_statesys.clone(), 
                                    tx_moc_client_statesys.clone(), 
                                    tx_pc_client_statesys.clone(), 
                                    tx_tc_client_statesys.clone(), 
                                    tx_cc_client_statesys.clone(), 
                                    updater_shared_player_missing_packets.clone(),
                                    buf_udp,
                                    packet_size
                                ).await;
                            } 
                            else
                            {
                                cli_log::info!("rejected: invalid session id");
                            }
                        }
                        else
                        {
                            cli_log::info!("rejected: client process should be handling this");
                        }
                    }
                }
                Some((socket, active_session_id)) = rx_addr_client_realtime.recv() => 
                {
                    cli_log::info!("removing entry from hash set");
                    server_state.online_players.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                    let mut clients_data = server_lock.lock().await;
                    let character_id = clients_data.get(&socket);
                    if let Some(session_id) = character_id.map(|(id, _faction)| &map.logged_in_players[*id as usize])
                    {
                        let current_session_id = session_id.load(std::sync::atomic::Ordering::Relaxed);
                        if current_session_id == active_session_id
                        {
                            session_id.store(0, std::sync::atomic::Ordering::Relaxed);
                            let _removed_player_id = clients_data.remove(&socket);
                        }
                        else
                        {
                            cli_log::info!("probably a reconnection {:?}", character_id);
                        }
                    }
                }
            }
        }   
    });

    (
        rx_mc_client_statesys,
        rx_moc_client_statesys,
        rx_pc_client_statesys,
        rx_tc_client_statesys,
        rx_cc_client_statesys,
        tx_bytes_statesys_socket
    )
}


