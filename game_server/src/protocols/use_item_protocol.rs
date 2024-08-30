use tokio::{sync::mpsc::Sender, net::UdpSocket};

use crate::character::character_command::{CharacterCommand, CharacterCommandInfo};


// we cant do the same is inventory request, because selling modifies the faction inventory and we need to propagate those changes.

pub async fn process(
     data : &[u8; 508],
    channel_player_tx : &Sender<CharacterCommand>)
{
        let mut start = 1;
        let mut end = start + 8;
        let _player_session_id = u64::from_le_bytes(data[start..end].try_into().unwrap());

        start = end;
        end = start + 2;
        let player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());

        start = end;
        end = start + 1;
        let faction = data[start];

        start = end;
        end = start + 4;
        let item_id = u32::from_le_bytes(data[start..end].try_into().unwrap()); 

        start = end;
        end = start + 2;
        let amount = u16::from_le_bytes(data[start..end].try_into().unwrap()); 

        let command = CharacterCommand{
            player_id,
            info: CharacterCommandInfo::UseItem(faction, item_id, amount)
        };

        println!("got a command {:?}", command);

        channel_player_tx.send(command).await.unwrap();
}