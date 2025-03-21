use tokio::{sync::mpsc::Sender, net::UdpSocket};

use crate::{character::character_command::{CharacterCommand, CharacterCommandInfo, CharacterMovement}, gaia_mpsc::GaiaSender, map::tetrahedron_id::TetrahedronId};


pub async fn process_respawn(
     data : &[u8; 508],
    channel_tx : &GaiaSender<CharacterCommand>)
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
    let tile_id = TetrahedronId::from_bytes(&buffer);

    let character_command = CharacterCommand
    {
        player_id,
        info: CharacterCommandInfo::Respawn(tile_id)
    };

    channel_tx.send(character_command).await.unwrap();
}