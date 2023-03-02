pub mod ping_protocol;
pub mod movement_protocol;
pub mod interaction_protocol;


use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

use crate::ServerState;
use crate::map::map_entity::MapCommand;
use crate::player::player_command::PlayerCommand;


pub enum Protocol{
    Ping = 1,
    Action = 2,
    GlobalState = 3,
    Interaction = 4,
}
    
pub async fn route_packet(
    socket: &UdpSocket,
    data : &[u8; 508],
    server_state: &Arc<ServerState>,
    channel_tx : &Sender<PlayerCommand>,
    channel_map_tx : &Sender<MapCommand>
){

    match data.get(0) {
        Some(protocol) if *protocol == Protocol::Ping as u8 => {
            ping_protocol::process_ping(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::Action as u8 => {
            let capacity = channel_tx.capacity();
            server_state.tx_pc_client_gameplay.store(capacity, std::sync::atomic::Ordering::Relaxed);
            movement_protocol::process_movement(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::Interaction as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity, std::sync::atomic::Ordering::Relaxed);
            interaction_protocol::process_interaction(socket, data, channel_map_tx).await;
        },
        _ => {
            println!("unknown protocol");
        }
    }
}
