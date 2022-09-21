use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

use crate::player_action::ClientAction;


pub async fn process_movement(_socket:&UdpSocket, data : &[u8; 508], channel_tx : &Sender<ClientAction>)
{
    let client_action = ClientAction::from_bytes(data);
    // println!("got a {:?}", client_action.position);
    channel_tx.send(client_action).await.unwrap();
}