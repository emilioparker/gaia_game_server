use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

use crate::map::map_entity::{MapCommand};


pub async fn process_interaction(_socket:&UdpSocket, data : &[u8; 508],  channel_map_tx : &Sender<MapCommand>)
{
    let map_action = MapCommand::from_bytes(data);
    println!("got a {:?} {:?}",map_action, map_action.id.to_string());
    channel_map_tx.send(map_action).await.unwrap();
}