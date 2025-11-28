use crate::{gaia_mpsc::GaiaSender, map::tetrahedron_id::TetrahedronId, mob::mob_command::{MobCommand, MobCommandInfo}};


pub async fn process(
     data : &[u8],
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

        let map_action = MobCommand
        {
            tile_id,
            info: MobCommandInfo::MoveMob(player_id, origin_position, end_position, path)
        };

        cli_log::info!("-------------------------- got a {:?}", map_action);

        channel_mob_tx.send(map_action).await.unwrap();
}