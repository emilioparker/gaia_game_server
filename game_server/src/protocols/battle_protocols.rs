use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

use crate::{battle::battle_command::{BattleCommand, BattleCommandInfo}, map::tetrahedron_id::TetrahedronId};


pub async fn process_join(_socket:&UdpSocket, data : &[u8; 508],  channel_battle_tx : &Sender<BattleCommand>)
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

    let info = BattleCommandInfo::Join();
    let map_action = BattleCommand {tile_id, player_id, info };
    
    channel_battle_tx.send(map_action).await.unwrap();
}


pub async fn process_turn(_socket:&UdpSocket, data : &[u8; 508],  channel_battle_tx : &Sender<BattleCommand>)
{
    println!(" process turn ---------- ");
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
    // end = start + 1;
    let participant_id = data[start];

    let info = BattleCommandInfo::Attack(participant_id);
    let map_action = BattleCommand {tile_id, player_id, info };
    
    channel_battle_tx.send(map_action).await.unwrap();
}