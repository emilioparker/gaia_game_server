use tokio::{sync::mpsc::Sender, net::UdpSocket};

use crate::{chat::ChatCommand, gaia_mpsc::GaiaSender, map::tetrahedron_id::TetrahedronId};


pub async fn process(
     data : &[u8],
    channel_tower_tx : &GaiaSender<ChatCommand>)
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

        let chat_message = ChatCommand
        {
            id: tile_id,
            faction,
            player_id,
            message_length,
            message
        };

        cli_log::info!("got a {:?}", chat_message);

        channel_tower_tx.send(chat_message).await.unwrap();
}