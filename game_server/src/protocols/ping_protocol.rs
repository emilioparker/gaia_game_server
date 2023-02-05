
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::player::player_action::PlayerAction;
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub async fn process_ping(socket:&UdpSocket, data : &[u8; 508], _channel_tx : &Sender<PlayerAction>)
{
    // let mut start = 1;
    // let mut end = start + 8;

    // let _player_id = u64::from_le_bytes(data[start..end].try_into().unwrap());
    // start = end;
    // end = start + 2;

    // // ping
    // let _num = u16::from_le_bytes(data[start..end].try_into().unwrap());
    // let original_data : [u8; 32] = [0,1,2,3,4,5,6,7,8,9,10,11,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1];

    let mut encoder = ZlibEncoder::new(Vec::new(),Compression::new(9));        
    std::io::Write::write_all(&mut encoder, data).unwrap();
    let compressed_bytes = encoder.reset(Vec::new()).unwrap();

    let _len = socket.send(&compressed_bytes).await.unwrap();

    // println!("{:?} bytes sent, original {:?}", _len, data.len());
}