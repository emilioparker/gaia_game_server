pub mod ping_protocol;
pub mod movement_protocol;
pub mod interaction_protocol;


use tokio::net::{UdpSocket, TcpStream};
use tokio::sync::mpsc::Sender;

use crate::map::map_entity::MapCommand;
use crate::player::player_action::PlayerAction;


pub enum Protocol{
    Ping = 1,
    Position = 2,
    GlobalState = 3,
    Interaction = 4,
}
    
pub async fn route_packet(
    data : &[u8; 508],
    channel_tx : &Sender<PlayerAction>,
    ping_channel_tx : &Sender<[u8;508]>,
    channel_map_tx : &Sender<MapCommand>
){

    match data.get(0) {
        Some(protocol) if *protocol == Protocol::Ping as u8 => {
            ping_protocol::process_ping(data, ping_channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::Position as u8 => {
            movement_protocol::process_movement(data, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::Interaction as u8 => {
            interaction_protocol::process_interaction(data, channel_map_tx).await;
        },
        _ => {
            println!("unknown protocol");
        }
    }
}
