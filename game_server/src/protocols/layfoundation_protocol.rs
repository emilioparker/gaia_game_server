use tokio::{sync::mpsc::Sender, net::UdpSocket};

use crate::map::{map_entity::{MapCommand, MapCommandInfo}, tetrahedron_id::TetrahedronId};


pub async fn process_construction(
     data : &[u8; 508],
    channel_map_tx : &Sender<MapCommand>)
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
        end = start + 1;
        let full_health = data[start];

        start = end;
        end = start + 2;
        let count = u16::from_le_bytes(data[start..end].try_into().unwrap()); 


        cli_log::info!("construction protocol count {}", count);

        start = end;
        for _ in 0..count {

            end = start + 6;
            let mut buffer = [0u8;6];
            buffer.copy_from_slice(&data[start..end]);
            let tile_id = TetrahedronId::from_bytes(&buffer);
            start = end;

            end = start + 4;
            let prop = u32::from_le_bytes(data[start..end].try_into().unwrap()); 
            start = end;

            end = start + 4;
            let pathness_a = f32::from_le_bytes(data[start..end].try_into().unwrap()); 
            start = end;

            end = start + 4;
            let pathness_b = f32::from_le_bytes(data[start..end].try_into().unwrap()); 
            start = end;

            end = start + 4;
            let pathness_c = f32::from_le_bytes(data[start..end].try_into().unwrap()); 
            start = end;

            let map_action = MapCommand{
                id: tile_id,
                info: MapCommandInfo::LayFoundation(player_id, prop, full_health, pathness_a, pathness_b, pathness_c)
            };

            // cli_log::info!("got a {:?}", map_action);

            channel_map_tx.send(map_action).await.unwrap();
        }
}