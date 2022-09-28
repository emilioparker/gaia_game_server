

use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

use crate::{ping_protocol, movement_protocol, player_action::PlayerAction, interaction_protocol};


pub const PING: u8 = 1;
pub const POSITION: u8 = 2; 
pub const GLOBAL_STATE: u8 = 3;
pub const INTERACTION: u8 = 4;

    
pub async fn route_packet(
    socket: &UdpSocket,
    data : &[u8; 508],
    channel_tx : &Sender<PlayerAction>
){

    match data.get(0) {
        Some(protocol) if *protocol == PING => {
            ping_protocol::process_ping(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == POSITION => {
            movement_protocol::process_movement(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == INTERACTION => {
            interaction_protocol::process_interaction(socket, data, channel_tx).await;
        },
        _ => {}
    }
}
