
use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::map::GameMap;
use crate::character::character_command::CharacterCommand;
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub async fn process_ping(
    socket:&UdpSocket, 
    data : &[u8; 508],
    map : Arc<GameMap>,
    _channel_tx : &Sender<CharacterCommand>)
{
    let mut start = 1;
    let end = start + 2;
    let id = u16::from_le_bytes(data[start..end].try_into().unwrap()); 

    let mut buffer = [0u8; 11];

    let time = &map.time.load(std::sync::atomic::Ordering::Relaxed);
    let time_bytes = u64::to_le_bytes(*time);

    let id_bytes = u16::to_le_bytes(id);


    let mut encoder = ZlibEncoder::new(Vec::new(),Compression::new(9));        
    buffer[0] = crate::protocols::Protocol::Ping as u8;

    let mut start = 1;
    let mut end = start + 2; 
    buffer[start..end].copy_from_slice(&id_bytes);
    start = end;

    end = start + 8;
    buffer[start..end].copy_from_slice(&time_bytes);
    start = end;

    std::io::Write::write_all(&mut encoder, &buffer).unwrap();
    let compressed_bytes = encoder.reset(Vec::new()).unwrap();

    let _len = socket.send(&compressed_bytes).await.unwrap();
}