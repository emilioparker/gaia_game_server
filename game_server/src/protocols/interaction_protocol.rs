use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

use crate::player::player_action::PlayerAction;


pub async fn process_interaction(_socket:&UdpSocket, data : &[u8; 508], channel_tx : &Sender<PlayerAction>)
{
    // let client_action = PlayerAction::from_bytes(data);
    // // println!("got a {:?}", client_action.position);
    // channel_tx.send(client_action).await.unwrap();
}