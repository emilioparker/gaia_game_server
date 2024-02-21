use tokio::{sync::mpsc::Sender, net::UdpSocket};

use crate::character::character_command::{CharacterCommand, CharacterCommandInfo, CharacterMovement};


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

    // 1 byte + 8 bytes + 1 byte + 4x3:12 bytes + 4x3:12 bytes + 4 bytes = 18 bytes
    start = end;
    end = start + 4;
    let pos_x = f32::from_le_bytes(data[start..end].try_into().unwrap());

    start = end;
    end = start + 4;
    let pos_y = f32::from_le_bytes(data[start..end].try_into().unwrap());

    start = end;
    end = start + 4;
    let pos_z = f32::from_le_bytes(data[start..end].try_into().unwrap());

    let position = [pos_x, pos_y, pos_z];

    start = end;
    end = start + 4;
    let direction_x = f32::from_le_bytes(data[start..end].try_into().unwrap());

    start = end;
    end = start + 4;
    let direction_y = f32::from_le_bytes(data[start..end].try_into().unwrap());

    start = end;
    end = start + 4;
    let direction_z = f32::from_le_bytes(data[start..end].try_into().unwrap());

    let direction = [direction_x, direction_y, direction_z];

    start = end;
    end = start + 2;
    let other_player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());
    start = end;

    end = start + 4;
    let action = u32::from_le_bytes(data[start..end].try_into().unwrap());
    start = end;

    end = start + 4;
    let required_time = u32::from_le_bytes(data[start..end].try_into().unwrap());
    //start = end;

    let action = CharacterMovement 
    {
        player_id,
        position,
        second_position: direction,
        other_player_id,
        action,
        required_time,
        skill_id: 0,
    };

    let character_command = CharacterCommand
    {
        player_id,
        info: CharacterCommandInfo::Movement(action)
    };

    channel_tx.send(character_command).await.unwrap();
}