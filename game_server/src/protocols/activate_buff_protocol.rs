use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

use crate::{character::character_command::{CharacterCommand, CharacterCommandInfo}, gaia_mpsc::GaiaSender};


pub async fn process(data : &[u8; 508],  channel_player_tx : &GaiaSender<CharacterCommand>)
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
    end = start + 4;
    let card_id = u32::from_le_bytes(data[start..end].try_into().unwrap()); // 4 bytes

    let command = CharacterCommand
    {
        player_id,
        info: CharacterCommandInfo::ActivateBuff(card_id)
    };

    cli_log::info!("got a command {:?}", command);

    channel_player_tx.send(command).await.unwrap();
}