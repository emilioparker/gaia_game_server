pub mod ping_protocol;
pub mod movement_protocol;
pub mod interaction_protocol;
pub mod inventory_request_protocol;


use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

use crate::ServerState;
use crate::map::GameMap;
use crate::map::map_entity::MapCommand;
use crate::player::player_command::PlayerCommand;


pub enum Protocol{
    Ping = 1,
    Action = 2,
    GlobalState = 3,
    Interaction = 4,
    InventoryRequest = 5,
}
    
pub async fn route_packet(
    player_id: u64,
    socket: &UdpSocket,
    data : &[u8; 508],
    map : Arc<GameMap>,
    server_state: &Arc<ServerState>,
    channel_tx : &Sender<PlayerCommand>,
    channel_map_tx : &Sender<MapCommand>
){

    match data.get(0) {
        Some(protocol) if *protocol == Protocol::Ping as u8 => {
            ping_protocol::process_ping(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::InventoryRequest as u8 => {
            inventory_request_protocol::process_request(player_id, socket, data, map, channel_tx).await;
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
        unknown_protocol => {
            println!("unknown protocol {:?}", unknown_protocol);
        }
    }
}
