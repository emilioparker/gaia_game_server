pub mod client_handler;
pub mod utils;
pub mod websocket_client_handler;

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, AtomicU64};
use std::collections::HashMap;
use crate::gaia_mpsc::GaiaSender;
use crate::gameplay_service::generic_command::GenericCommand;
use crate::mob::mob_command::MobCommand;
use crate::{gaia_mpsc, ServerChannels, ServerState};
use crate::chat::ChatCommand;
use crate::map::GameMap;
use crate::map::map_entity::MapCommand;
use crate::hero::hero_command::HeroCommand;
use crate::tower::TowerCommand;
use bytes::Bytes;
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
    Receiver<HeroCommand>, 
    Receiver<TowerCommand>, 
    Receiver<ChatCommand>,
    GaiaSender<Vec<(u64,u8,u16,u32,Bytes)>>
) // packet number, faction, region, gamepackets,data
{
    let (tx_gc_clients_gameplay, mut rx_gc_clients_gameplay) = gaia_mpsc::channel::<GenericCommand>(100, ServerChannels::TX_GC_ClIENTS_GAMEPLAY, server_state.clone());
    let (tx_mc_clients_gameplay, rx_mc_clients_gameplay) = gaia_mpsc::channel::<MapCommand>(100, ServerChannels::TX_MC_CLIENTS_GAMEPLAY, server_state.clone());
    let (tx_moc_clients_gameplay, rx_moc_clients_gameplay) = gaia_mpsc::channel::<MobCommand>(100, ServerChannels::TX_MOC_CLIENTS_GAMEPLAY, server_state.clone());
    let (tx_pc_clients_gameplay, rx_pc_clients_gameplay) = gaia_mpsc::channel::<HeroCommand>(100, ServerChannels::TX_PC_CLIENTS_GAMEPLAY, server_state.clone());
    let (tx_tc_clients_gameplay, rx_tc_clients_gameplay) = gaia_mpsc::channel::<TowerCommand>(100, ServerChannels::TX_TC_CLIENTS_GAMEPLAY, server_state.clone());
    let (tx_cc_clients_gameplay, rx_cc_clients_gameplay) = gaia_mpsc::channel::<ChatCommand>(100, ServerChannels::TX_CC_CLIENTS_GAMEPLAY, server_state.clone());
    let (tx_packets_gameplay_chat_clients, mut rx_packets_gameplay_chat_clients) = gaia_mpsc::channel::<Vec<(u64, u8, u16, u32, Bytes)>>(100, ServerChannels::TX_PACKETS_GAMEPLAY_CHAT_CLIENTS, server_state.clone());

    let packet_builder_server_state = server_state.clone();
    let generic_packet_builder_server_state: Arc<ServerState> = server_state.clone();

    // udp connections
    let udp_client_connections:HashMap<std::net::SocketAddr, (u16, u8)> = HashMap::new();
    let udp_client_connections_mutex = std::sync::Arc::new(Mutex::new(udp_client_connections));

    let udp_client_connections_receiver_lock = udp_client_connections_mutex.clone();
    let udp_client_connections_sender_lock = udp_client_connections_mutex.clone();

    let udp_address: std::net::SocketAddr = "0.0.0.0:11004".parse().unwrap();
    let udp_socket = Arc::new(utils::create_reusable_udp_socket(udp_address));
    let send_udp_socket = udp_socket.clone();
    let send_directly_udp_socket = udp_socket.clone();
    
    let (tx_packets_gameplay_chat_websocket_clients, rx_packets_gameplay_chat_websocket_clients) =  gaia_mpsc::channel::<Vec<(u64, u8, u16, u32, Bytes)>>(100, ServerChannels::TX_PACKETS_GAMEPLAY_CHAT_WEBSOCKET_CLIENTS, server_state.clone());
    let (tx_packets_gameplay_chat_websocket_specific_client, rx_packets_gameplay_chat_websocket_specific_clients) =  gaia_mpsc::channel::<(SocketAddr, Bytes)>(100, ServerChannels::TX_PACKETS_GAMEPLAY_CHAT_WEBSOCKET_SPECIFIC_CLIENT, server_state.clone());


    let mut player_regions_record = HashMap::<u16, [AtomicU16;3]>::new();

    let mut i:u16 = 0;

    while i < u16::MAX 
    {
        i = i + 1;
        let data = 
        [
            AtomicU16::new(0),
            AtomicU16::new(0),
            AtomicU16::new(0),
        ];
        player_regions_record.insert(i, data);
    }

    let shared_player_regions_record= Arc::new(player_regions_record);
    let reader_shared_player_regions_record= shared_player_regions_record.clone();
    let updater_shared_player_regions_record = reader_shared_player_regions_record.clone();

    let map_for_websocket = map.clone();
    let server_state_for_websocket = server_state.clone();
    let tx_gc_clients_gameplay_for_websocket = tx_gc_clients_gameplay.clone();
    let tx_mc_clients_gameplay_for_websocket = tx_mc_clients_gameplay.clone();
    let tx_moc_clients_gameplay_for_websocket = tx_moc_clients_gameplay.clone();
    let tx_pc_clients_gameplay_for_websocket = tx_pc_clients_gameplay.clone();
    let tx_tc_clients_gameplay_for_websocket = tx_tc_clients_gameplay.clone();
    let tx_cc_clients_gameplay_for_websocket = tx_cc_clients_gameplay.clone();
    let updater_shared_player_regions_record_for_websocket = updater_shared_player_regions_record.clone();

    tokio::spawn(async move 
    {
        websocket_client_handler::run(
        rx_packets_gameplay_chat_websocket_clients,
        rx_packets_gameplay_chat_websocket_specific_clients,
                map_for_websocket,
                server_state_for_websocket,
                tx_gc_clients_gameplay_for_websocket,
                tx_mc_clients_gameplay_for_websocket,
                tx_moc_clients_gameplay_for_websocket,
                tx_pc_clients_gameplay_for_websocket,
                tx_tc_clients_gameplay_for_websocket,
                tx_cc_clients_gameplay_for_websocket,
                updater_shared_player_regions_record_for_websocket
            ).await;
    });

    tokio::spawn(async move 
    {
        loop 
        {
            if let Some(command) = rx_gc_clients_gameplay.recv().await 
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

                generic_packet_builder_server_state.sent_bytes.fetch_add(command.data.len() as u64, std::sync::atomic::Ordering::Relaxed);
                generic_packet_builder_server_state.sent_udp_packets.fetch_add(1u64, std::sync::atomic::Ordering::Relaxed);
                generic_packet_builder_server_state.sent_game_packets.fetch_add(1u64, std::sync::atomic::Ordering::Relaxed);

                tx_packets_gameplay_chat_websocket_specific_client.send((command.player_address, command.data)).await;
            }
        }
    });

    tokio::spawn(async move 
    {
        loop 
        {
            // there are two sources of packets, chat and game. Each one has a differente packet id.
            if let Some(packet_list) = rx_packets_gameplay_chat_clients.recv().await 
            {
                // let mut first_client = true;
                let mut sent_bytes : u64 = 0;
                let mut sent_game_packets : u64 = 0;
                let mut sent_udp_packets : u64 = 0;
                let mut clients_data = udp_client_connections_sender_lock.lock().await;
                for client in clients_data.iter_mut()
                {
                    for (_packet_id, faction, region, game_packets, data) in packet_list.iter()
                    {
                        // cli_log::info!("sending packet with id {packet_id}");
                        if client.1.0 == 0u16 
                        {
                            // let first_byte = packet[0]; // this is the protocol
                            // the packet is compress, I can't read the sequence number
                            // let packet_sequence_number = u64::from_le_bytes(packet[1..9].try_into().unwrap());
                            // cli_log::info!("sending {}", packet_sequence_number);
                        }
                        // todo: only send data if client is correctly validated, add state to clients_data
                        
                        let client_regions = reader_shared_player_regions_record.get(&client.1.0).unwrap();
                        let a = client_regions[0].load(std::sync::atomic::Ordering::Relaxed);
                        let b = client_regions[1].load(std::sync::atomic::Ordering::Relaxed);
                        let c = client_regions[2].load(std::sync::atomic::Ordering::Relaxed);

                        // a = 0 makes us receive everything
                        let is_in_region = *region == a || *region == b || *region == c || a == 0 || *region == 0;

                        if is_in_region && (client.1.1 == *faction || *faction == 0)
                        {
                            sent_bytes += data.len() as u64;
                            sent_udp_packets += 1;
                            sent_game_packets += *game_packets as u64;
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
                    }

                    packet_builder_server_state.sent_bytes.fetch_add(sent_bytes, std::sync::atomic::Ordering::Relaxed);
                    packet_builder_server_state.sent_udp_packets.fetch_add(sent_udp_packets, std::sync::atomic::Ordering::Relaxed);
                    packet_builder_server_state.sent_game_packets.fetch_add(sent_game_packets, std::sync::atomic::Ordering::Relaxed);
                }

                tx_packets_gameplay_chat_websocket_clients.send(packet_list).await;
            }
        }
    });

    tokio::spawn(async move 
    {
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
                        let mut clients_data = udp_client_connections_receiver_lock.lock().await;
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
                                    udp_address, 
                                    from_address, 
                                    map.clone(),
                                    server_state.clone(),
                                    tx_gc_clients_gameplay.clone(),
                                    tx_addr_client_realtime.clone(), 
                                    tx_mc_clients_gameplay.clone(), 
                                    tx_moc_clients_gameplay.clone(), 
                                    tx_pc_clients_gameplay.clone(), 
                                    tx_tc_clients_gameplay.clone(), 
                                    tx_cc_clients_gameplay.clone(), 
                                    updater_shared_player_regions_record.clone(),
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
                    let mut clients_data = udp_client_connections_receiver_lock.lock().await;
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
        rx_mc_clients_gameplay,
        rx_moc_clients_gameplay,
        rx_pc_clients_gameplay,
        rx_tc_clients_gameplay,
        rx_cc_clients_gameplay,
        tx_packets_gameplay_chat_clients
    )
}


