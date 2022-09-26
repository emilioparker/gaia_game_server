use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::player_action::PlayerAction;

pub async fn process_ping(socket:&UdpSocket, data : &[u8; 508], _channel_tx : &Sender<PlayerAction>)
{
    let mut start = 1;
    let mut end = start + 8;

    let player_id = u64::from_le_bytes(data[start..end].try_into().unwrap());
    start = end;
    end = start + 2;

    // ping
    let _num = u16::from_le_bytes(data[start..end].try_into().unwrap());


    // println!("the message is an {num}");
    let _len = socket.send(data).await.unwrap();
    // println!("{:?} bytes sent", len);
}