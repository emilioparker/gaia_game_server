use crate::{gaia_mpsc::GaiaSender, map::tetrahedron_id::TetrahedronId, mob::mob_command::{MobCommand, SpawnMobData}};


pub async fn process(
     data : &[u8],
    channel_map_tx : &GaiaSender<MobCommand>)
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
        let mob_definition_id = u32::from_le_bytes(data[start..end].try_into().unwrap()); 
        start = end;

        end = start + 1;
        let level = data[start]; 
        start = end;

        let map_action = MobCommand::Spawn(SpawnMobData
        {
            hero_id: player_id,
            mob_definition_id,
            tile_id,
            level,
        });

        channel_map_tx.send(map_action).await.unwrap();
}