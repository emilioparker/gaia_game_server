use tokio::{sync::mpsc::Sender, net::UdpSocket};

use crate::{tower::{TowerCommand, TowerCommandInfo}, map::tetrahedron_id::TetrahedronId, chat::ChatCommand};


pub async fn process(
    _socket:&UdpSocket,
     data : &[u8; 508],
    channel_tower_tx : &Sender<ChatCommand>)
{
        let mut start = 1;
        let mut end = start + 8;
        let player_session_id = u64::from_le_bytes(data[start..end].try_into().unwrap());

        start = end;
        end = start + 2;
        let player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());

        start = end;
        end = start + 1;
        let faction = data[start];

        start = end; // ignoring first byte
        end = start + 6;
        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let tile_id = TetrahedronId::from_bytes(&buffer);
        start = end;

        end = start + 1;
        let message_length = data[start]; 
        start = end;

        let mut message = [0u32; 100];

        for i in 0..usize::min(100, message_length as usize)
        {
            end = start + 4;
            let letter = u32::from_le_bytes(data[start..end].try_into().unwrap());
            start = end;
            message[i] = letter;
        }

        let chat_message = ChatCommand{
            id: tile_id,
            faction,
            player_id,
            message_length,
            message
        };

        // println!("got a {:?}", chat_message);

        channel_tower_tx.send(chat_message).await.unwrap();
}