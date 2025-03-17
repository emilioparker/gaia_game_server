use crate::{gaia_mpsc::GaiaSender, map::tetrahedron_id::TetrahedronId, mob::mob_command::MobCommand};


pub async fn process(
     data : &[u8; 508],
    channel_mob_tx : &GaiaSender<MobCommand>)
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

        // current tile, it must be a mob
        start = end;
        end = start + 6;
        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let tile_id = TetrahedronId::from_bytes(&buffer);
        start = end;

        // new tile, must be empty and not water.
        end = start + 6;
        buffer.copy_from_slice(&data[start..end]);
        let new_tile_id = TetrahedronId::from_bytes(&buffer);
        start = end;

        end = start + 4;
        let mob_id = u32::from_le_bytes(data[start..end].try_into().unwrap()); 
        start = end;

        end = start + 4;
        let distance = f32::from_le_bytes(data[start..end].try_into().unwrap()); 
        start = end;
        
        end = start + 4;
        let required_time = f32::from_le_bytes(data[start..end].try_into().unwrap()); 
        // start = end;

        // let map_action = MobCommand{
        //     tile_id,
        //     info: MobCommandInfo::MoveMob(player_id, mob_id, new_tile_id, distance, required_time)
        // };

        // // cli_log::info!("got a {:?}", map_action);

        // channel_map_tx.send(map_action).await.unwrap();
}