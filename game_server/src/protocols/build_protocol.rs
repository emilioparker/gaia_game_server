use crate::{gaia_mpsc::GaiaSender, map::{map_entity::{MapCommand, MapCommandInfo}, tetrahedron_id::TetrahedronId}};


pub async fn process(
     data : &[u8],
    channel_map_tx : &GaiaSender<MapCommand>)
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
        end = start + 6;
        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let tile_id = TetrahedronId::from_bytes(&buffer);
        start = end;

        end = start + 4;
        let increment = u32::from_le_bytes(data[start..end].try_into().unwrap()); 
        // start = end;

        let map_action = MapCommand{
            id: tile_id,
            info: MapCommandInfo::BuildStructure(player_id, increment)
        };

        // cli_log::info!("got a {:?}", map_action);

        channel_map_tx.send(map_action).await.unwrap();
}