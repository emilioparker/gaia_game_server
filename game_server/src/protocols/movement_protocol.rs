use tokio::{sync::mpsc::Sender, net::UdpSocket};

use crate::{character::character_command::{CharacterCommand, CharacterCommandInfo, CharacterMovement}, map::tetrahedron_id::TetrahedronId};


pub async fn process_movement(
    _socket:&UdpSocket,
     data : &[u8; 508],
    channel_tx : &Sender<CharacterCommand>)
{
    //1 - protocolo 1 bytes
    //2 - id 8 bytes
    // the rest depends on the code.
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
    let position_tile_id = TetrahedronId::from_bytes(&buffer);
    start = end;

    end = start + 6;
    let mut buffer = [0u8;6];
    buffer.copy_from_slice(&data[start..end]);
    let second_position_tile_id = TetrahedronId::from_bytes(&buffer);
    start = end;

    end = start + 4;
    let vertex_id = i32::from_le_bytes(data[start..end].try_into().unwrap());
    start = end;

    let mut path : [u8;6] = [0,0,0,0,0,0];
    for i in 0..6
    {
        end = start + 1;
        path[i] = data[start];
        start = end;
    }

    end = start + 4;
    let start_time = u32::from_le_bytes(data[start..end].try_into().unwrap());
    start = end;

    let action = CharacterMovement 
    {
        player_id,
        position: position_tile_id,
        second_position: second_position_tile_id,
        vertex_id,
        path,
        time:start_time
    };

    let character_command = CharacterCommand
    {
        player_id,
        info: CharacterCommandInfo::Movement(action)
    };

    channel_tx.send(character_command).await.unwrap();
}