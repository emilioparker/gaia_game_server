use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

use crate::{hero::hero_command::{HeroCommand, HeroCommandInfo}, gaia_mpsc::GaiaSender};


pub async fn process(data : &[u8; 508],  channel_character_tx : &GaiaSender<HeroCommand>)
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

    end = start + 2;
    let other_player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());
    start = end;

    end = start + 4;
    let card_id = u32::from_le_bytes(data[start..end].try_into().unwrap()); // 4 bytes
    start = end;

    end = start + 4;
    let required_time = u32::from_le_bytes(data[start..end].try_into().unwrap()); // 4 bytes
    start = end;

    end = start + 1;
    let active_effect = data[start]; // 1 bytes
    start = end;

    end = start + 1;
    let missed = data[start]; // 1 bytes
    // start = end;

    let info = HeroCommandInfo::AttackCharacter(other_player_id, card_id, required_time, active_effect, missed);
    let map_action = HeroCommand { player_id, info };
    
    channel_character_tx.send(map_action).await.unwrap();
}