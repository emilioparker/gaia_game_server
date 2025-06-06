use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

use crate::{gaia_mpsc::GaiaSender, map::{map_entity::{MapCommand, MapCommandInfo}, tetrahedron_id::TetrahedronId}};


pub async fn process(data : &[u8],  channel_map_tx : &GaiaSender<MapCommand>)
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

    start = end; // ignoring first byte
    end = start + 6;
    let mut buffer = [0u8;6];
    buffer.copy_from_slice(&data[start..end]);
    let tile_id = TetrahedronId::from_bytes(&buffer);

    start = end;
    end = start + 2;
    let damage = u16::from_le_bytes(data[start..end].try_into().unwrap()); // 2 bytes

    let info = MapCommandInfo::ResourceExtraction(player_id, damage);
    let map_action = MapCommand { id: tile_id, info };
    
    // let map_action = MapCommand::from_bytes(data);
    // cli_log::info!("got a {:?} {:?}",map_action, map_action.id.to_string());
    channel_map_tx.send(map_action).await.unwrap();
}