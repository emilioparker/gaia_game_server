use tokio::{sync::mpsc::Sender, net::UdpSocket};

use crate::character::character_command::CharacterMovement;


pub async fn process(
    _socket:&UdpSocket,
     data : &[u8; 508],
    channel_player_tx : &Sender<CharacterMovement>)
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
        let item_id = u32::from_le_bytes(data[start..end].try_into().unwrap()); 

        start = end;
        end = start + 2;
        let amount = u16::from_le_bytes(data[start..end].try_into().unwrap()); 

        // let map_action = MapCommand{
        //     id: tile_id,
        //     info: MapCommandInfo::BuildStructure(player_id, increment)
        // };

        // // println!("got a {:?}", map_action);

        // channel_map_tx.send(map_action).await.unwrap();
}