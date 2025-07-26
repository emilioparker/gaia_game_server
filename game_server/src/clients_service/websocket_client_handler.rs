use axum::http::version;
use tokio::{net::{TcpListener, TcpStream}, sync::{broadcast, mpsc::{self, Receiver, Sender}, watch, Mutex}, time};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use futures_util::{stream::ForEach, SinkExt, StreamExt}; // for reading/writing messages
use std::{collections::{vec_deque, HashMap}, net::SocketAddr, sync::{atomic::{AtomicBool, AtomicU16}, Arc}, time::Duration};
use bytes::Bytes;

use crate::{chat::ChatCommand, gaia_mpsc, gameplay_service::generic_command::GenericCommand, hero::hero_command::HeroCommand, kingdom::KingdomCommand, map::{map_entity::MapCommand, GameMap}, mob::mob_command::MobCommand, protocols, tower::TowerCommand, ServerState};

pub struct WebSocketConnection
{
    pub session_id: u64,
    pub hero_id : u16,
    pub faction : u8,
    pub address : SocketAddr,
    pub link : tokio::sync::mpsc::Sender<Bytes>
    // pub link : futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>
}


pub async fn run(
    from_server:  tokio::sync::mpsc::Receiver<Vec<(u64, u8, u16, u32, Bytes)>>,
    from_server_specific:  tokio::sync::mpsc::Receiver<(SocketAddr, Bytes)>,
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    tx_gc_clients_gameplay : gaia_mpsc::GaiaSender<GenericCommand>,
    // diconnected_channel_tx : mpsc::Sender<(std::net::SocketAddr, u64)>,
    tx_mc_clients_gameplay : gaia_mpsc::GaiaSender<MapCommand>,
    tx_moc_clients_gameplay : gaia_mpsc::GaiaSender<MobCommand>,
    tx_pc_clients_gameplay : gaia_mpsc::GaiaSender<HeroCommand>,
    tx_tc_clients_gameplay : gaia_mpsc::GaiaSender<TowerCommand>,
    tx_kc_clients_gameplay : gaia_mpsc::GaiaSender<KingdomCommand>,
    tx_cc_clients_gameplay : gaia_mpsc::GaiaSender<ChatCommand>,
    regions : Arc<HashMap<u16, [AtomicU16;3]>>)
{
    // Bind to a local TCP socket
    let addr = "0.0.0.0:11005";
    let listener = TcpListener::bind(&addr).await.expect("Can't bind");
    cli_log::info!("WebSocket server running at ws://{}", addr);

    let clients = Arc::new(Mutex::new(HashMap::new()));

    tokio::spawn(send_data_to_clients(from_server, clients.clone(), regions.clone()));
    tokio::spawn(send_data_to_specific_client(from_server_specific, clients.clone()));

    // Accept incoming connections
    while let Ok((stream, socket_addr)) = listener.accept().await 
    {
        let clients_for_adding= clients.clone();
        tokio::spawn(handle_connection(
            stream,
            socket_addr,
            map.clone(),
            server_state.clone(),
            clients_for_adding,
            tx_gc_clients_gameplay.clone(),
            tx_mc_clients_gameplay.clone(),
            tx_moc_clients_gameplay.clone(),
            tx_pc_clients_gameplay.clone(), 
            tx_tc_clients_gameplay.clone(),
            tx_kc_clients_gameplay.clone(),
            tx_cc_clients_gameplay.clone(),
            regions.clone()
        ));
    }
}


async fn handle_connection(
    stream: tokio::net::TcpStream,
    addr: SocketAddr,
    // hero_id: u16,
    // faction:u8,
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    clients: Arc<Mutex<HashMap<SocketAddr, WebSocketConnection>>>,
    tx_gc_clients_gameplay : gaia_mpsc::GaiaSender<GenericCommand>,
    // diconnected_channel_tx : mpsc::Sender<(std::net::SocketAddr, u64)>,
    tx_mc_clients_gameplay : gaia_mpsc::GaiaSender<MapCommand>,
    tx_moc_clients_gameplay : gaia_mpsc::GaiaSender<MobCommand>,
    tx_pc_clients_gameplay : gaia_mpsc::GaiaSender<HeroCommand>,
    tx_tc_clients_gameplay : gaia_mpsc::GaiaSender<TowerCommand>,
    tx_kc_clients_gameplay : gaia_mpsc::GaiaSender<KingdomCommand>,
    tx_cc_clients_gameplay : gaia_mpsc::GaiaSender<ChatCommand>,
    regions : Arc<HashMap<u16, [AtomicU16;3]>>)
{
    cli_log::info!("New connection from {}", addr);

    // Perform the WebSocket handshake
    let ws_stream = accept_async(stream)
        .await
        .expect("WebSocket handshake failed");

    cli_log::info!("WebSocket connection established: {}", addr);

    // Split into sender and receiver
    let (write, mut read) = ws_stream.split();
    let (kill_tx, mut kill_rx) = tokio::sync::watch::channel(1);


    let (tx, rx) = tokio::sync::mpsc::channel::<Bytes>(100);
    tokio::spawn(send_data_to_client(rx, write, kill_tx));

    server_state.online_players.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let created = AtomicBool::new(false);

    'main_loop : loop
    {
        let time_out = time::sleep(Duration::from_secs_f32(10.0)); 
        tokio::select! 
        {
            _ = kill_rx.changed() => 
            {
                break 'main_loop;
            },
            _ = time_out => 
            {
                cli_log::info!("we couldn't wait any longer sorry!");
                break 'main_loop;
            }
            Some(msg) = read.next() =>
            {
                match msg 
                {
                    Ok(msg) => 
                    {
                        if msg.is_binary() 
                        {
                            let data = msg.into_data();

                            if !created.load(std::sync::atomic::Ordering::Relaxed)
                            {
                                cli_log::info!("websocket:creating client");
                                created.store(true, std::sync::atomic::Ordering::Relaxed);

                                let start = 1;
                                let end = start + 8;
                                let player_session_id = u64::from_le_bytes(data[start..end].try_into().unwrap());

                                let start = end;
                                let end = start + 2;
                                let player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());

                                let start = end;
                                let _end = start + 1;
                                let faction = data[start];

                                cli_log::info!("creating new websocket connection for {player_session_id} and hero id : {player_id}");

                                let mut clients_lock = clients.lock().await;
                                clients_lock.insert(addr, WebSocketConnection
                                {
                                    session_id : player_session_id,
                                    address: addr,
                                    link: tx.clone(),
                                    hero_id: player_id,
                                    faction,
                                });
                                drop(clients_lock);
                            }

                            // cli_log::info!("websocket:got data from client {}", data.len());
                            // let _result = to_server.send(msg.into_data()).await;
                            protocols::route_packet(
                                addr,
                                &data,
                                data.len(),
                                &map,
                                &server_state,
                                &regions,
                                &tx_gc_clients_gameplay,
                                &tx_pc_clients_gameplay, 
                                &tx_mc_clients_gameplay,
                                &tx_moc_clients_gameplay,
                                &tx_tc_clients_gameplay,
                                &tx_kc_clients_gameplay,
                                &tx_cc_clients_gameplay,
                            ).await;
                        }
                        else if msg.is_close() 
                        {
                            break 'main_loop;
                        }
                    }
                    Err(e) => 
                    {
                        cli_log::error!("Error processing connection: {}", e);
                        break 'main_loop;
                    }
                }
            }
        }
    }

    let mut clients_lock = clients.lock().await;
    clients_lock.remove(&addr);

    server_state.online_players.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    cli_log::info!("Connection {} closed", addr);
}

async fn send_data_to_client(
    mut from_server : tokio::sync::mpsc::Receiver<Bytes>,
    mut link : futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
    kill_watch : tokio::sync::watch::Sender<u8>
) 
{
    'main : loop 
    {
        if let Some(message) = from_server.recv().await
        {
            // cli_log::info!("websocket:sending data to specific client websocket <<<<<<----- this one {}" , message.len());
            let result = link.feed(Message::Binary(message)).await;
            if result.is_err()
            {
                cli_log::error!("error sending data to write link in websocket");
                break 'main;
            }

            let result = link.flush().await;
            if result.is_err()
            {
                cli_log::error!("websocket:error flushing data to write link in websocket");
                break;
            }
        }
        else
        { 
            cli_log::error!("websocket:error receiving data before sending to websocket");
            break;
        }
    }

    let _ = kill_watch.send(0);
}

async fn send_data_to_clients(
    mut from_server : tokio::sync::mpsc::Receiver<Vec<(u64, u8, u16, u32, Bytes)>>,
    clients: Arc<Mutex<HashMap<SocketAddr, WebSocketConnection>>>,
    regions : Arc<HashMap<u16, [AtomicU16;3]>>)
{
    loop 
    {
        if let Some(packet_list) = from_server.recv().await
        {
            let locked_clients = clients.lock().await;
            for client in locked_clients.iter()
            {
                // cli_log::info!("sending data to clients {}" , client.0);
                let client_regions = regions.get(&client.1.hero_id).unwrap();
                let a = client_regions[0].load(std::sync::atomic::Ordering::Relaxed);
                let b = client_regions[1].load(std::sync::atomic::Ordering::Relaxed);
                let c = client_regions[2].load(std::sync::atomic::Ordering::Relaxed);

                for (_packet_id, faction, region, game_packets, data) in packet_list.iter()
                {
                    let is_in_region = *region == a || *region == b || *region == c || a == 0 || *region == 0;
                    if is_in_region && (client.1.faction == *faction || *faction == 0)
                    {
                        // sent_bytes += data.len() as u64;
                        // sent_udp_packets += 1;
                        // sent_game_packets += *game_packets as u64;
                        // let result = send_udp_socket.try_send_to(data, client.0.clone());
                        let result = client.1.link.send(data.clone()).await;
                        match result 
                        {
                            Ok(_) => 
                            {
                                // cli_log::info!("data sent to client {}", packet.len());
                            },
                            Err(_) => cli_log::info!("error sending data to client queue"),
                        }
                    }
                }
            }
        }
        else
        { 
            cli_log::error!("error receiving data before sending to websocket");
            break;
        }
    }
}

async fn send_data_to_specific_client(
    mut from_server : tokio::sync::mpsc::Receiver<(SocketAddr, Bytes)>,
    clients: Arc<Mutex<HashMap<SocketAddr, WebSocketConnection>>>
) 
{
    'main : loop 
    {
        if let Some(message) = from_server.recv().await
        {
            cli_log::info!("websocket:sending data to specific client channel");
            let locked_clients = clients.lock().await;

            if let Some(client) = locked_clients.get(&message.0)
            {
                let _result = client.link.send(message.1).await;
            }
            else
            {
                cli_log::error!("client not found for direct message");
            }
        }
        else
        { 
            cli_log::error!("error receiving data before sending to websocket");
            break 'main;
        }
    }
}