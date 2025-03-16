use tokio::{sync::mpsc::Sender, net::UdpSocket};

use crate::{gaia_mpsc::GaiaSender, map::{map_entity::{MapCommand, MapCommandInfo}, tetrahedron_id::TetrahedronId}, mob::mob_command::{MobCommand, MobCommandInfo}};


pub async fn process(
     data : &[u8; 508],
    channel_map_tx : &GaiaSender<MobCommand>)
{
        let mut start = 1;
        let mut end = start + 8;
        let _player_session_id = u64::from_le_bytes(data[start..end].try_into().unwrap());

        start = end;
        end = start + 2;
        let _player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());

        start = end;
        end = start + 1;
        let _faction = data[start];
        
        start = end;
        end = start + 6;
        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let tile_id = TetrahedronId::from_bytes(&buffer);
        // start = end;

        let map_action = MobCommand
        {
            tile_id,
            info: MobCommandInfo::Touch(),
        };

        channel_map_tx.send(map_action).await.unwrap();
}