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
        let faction = data[start];

        // id
        start = end;
        end = start + 6;
        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let tile_id = TetrahedronId::from_bytes(&buffer);

        // endpointA
        start = end;
        end = start + 6;
        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let endpoint_a = TetrahedronId::from_bytes(&buffer);

        // endpointB
        start = end;
        end = start + 6;
        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let endpoint_b = TetrahedronId::from_bytes(&buffer);

        start = end;
        end = start + 1;
        let wall_size = data[start]; 

        start = end;
        end = start + 4;
        let prop = u32::from_le_bytes(data[start..end].try_into().unwrap()); 
        // println!("construction protocol count {}", count);

        let map_action = MapCommand{
            id: tile_id,
            info: MapCommandInfo::LayWallFoundation(player_id, faction, prop, endpoint_a, endpoint_b, wall_size)
        };

        println!("got a {:?}", map_action);

        channel_map_tx.send(map_action).await.unwrap();
}