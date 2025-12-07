use crate::{gaia_mpsc::GaiaSender, map::tetrahedron_id::TetrahedronId, mob::mob_command::{MobCommand, MoveMobData}};


pub async fn process(
     data : &[u8],
    channel_mob_tx : &GaiaSender<MobCommand>)
{
        let mut buffer = [0u8;6];
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
        let mob_id = u32::from_le_bytes(data[start..end].try_into().unwrap());
        start = end;

        // new tile, must be empty and not water.
        end = start + 6;
        buffer.copy_from_slice(&data[start..end]);
        let origin_position = TetrahedronId::from_bytes(&buffer);
        start = end;

        // new tile, must be empty and not water.
        end = start + 6;
        buffer.copy_from_slice(&data[start..end]);
        let end_position = TetrahedronId::from_bytes(&buffer);
        start = end;

        // path should point to the end position for consistency
        let mut path : [u8;6] = [0,0,0,0,0,0];
        for i in 0..6
        {
            end = start + 1;
            path[i] = data[start];
            start = end;
        }

        let map_action = MobCommand::MoveMob(MoveMobData
        {
            hero_id: player_id,
            mob_id,
            new_origin_tile_id: origin_position,
            new_end_tile_id: end_position,
            path,
        });

        cli_log::info!("-------------------------- got a {:?}", map_action);

        channel_mob_tx.send(map_action).await.unwrap();
}