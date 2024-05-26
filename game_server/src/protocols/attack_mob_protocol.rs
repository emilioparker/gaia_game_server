use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

use crate::map::{map_entity::{MapCommand, MapCommandInfo}, tetrahedron_id::TetrahedronId};


pub async fn process(_socket:&UdpSocket, data : &[u8; 508],  channel_map_tx : &Sender<MapCommand>)
{
    let mut start = 1;
    let mut end = start + 8;
    let _player_session_id = u64::from_le_bytes(data[start..end].try_into().unwrap());

    start = end;
    end = start + 2;
    let player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());

    start = end;
    end = start + 1;
    let _faction = data[start];

    start = end;
    end = start + 6;

    let mut buffer = [0u8;6];
    buffer.copy_from_slice(&data[start..end]);
    let tile_id = TetrahedronId::from_bytes(&buffer);

    start = end;
    end = start + 4;
    let card_id = u32::from_le_bytes(data[start..end].try_into().unwrap()); // 4 bytes

    start = end;
    end = start + 4;
    let required_time = u32::from_le_bytes(data[start..end].try_into().unwrap()); // 4 bytes

    start = end;
    let active_effect = data[start]; // 4 bytes

    print!("active effect {active_effect}");

    let info = MapCommandInfo::AttackMob(player_id, card_id, required_time, active_effect);
    let map_action = MapCommand { id: tile_id, info };
    
    // let map_action = MapCommand::from_bytes(data);
    channel_map_tx.send(map_action).await.unwrap();
}