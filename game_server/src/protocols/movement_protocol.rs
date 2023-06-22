use tokio::{sync::mpsc::Sender, net::UdpSocket};

use crate::character::character_command::CharacterCommand;


pub async fn process_movement(
    _socket:&UdpSocket,
     data : &[u8; 508],
    channel_tx : &Sender<CharacterCommand>)
{
    let client_action = CharacterCommand::from_bytes(data);
    // println!("got a {:?} {:?}", client_action.position, client_action.action);
    channel_tx.send(client_action).await.unwrap();
}