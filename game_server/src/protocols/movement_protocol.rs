use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

use crate::ServerState;
use crate::player::player_command::PlayerCommand;


pub async fn process_movement(
    _socket:&UdpSocket,
     data : &[u8; 508],
    channel_tx : &Sender<PlayerCommand>)
{
    let client_action = PlayerCommand::from_bytes(data);
    // println!("got a {:?}", client_action.position);
    channel_tx.send(client_action).await.unwrap();
}