use crate::{gaia_mpsc::GaiaSender, map::tetrahedron_id::TetrahedronId, mob::mob_command::{MobCommand, MobToHeroData}};


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
        start = end; // ignoring first byte

        end = start + 4;
        let attacker_mob_id = u32::from_le_bytes(data[start..end].try_into().unwrap()); // 4 bytes
        start = end;

        end = start + 6;
        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let tile_id = TetrahedronId::from_bytes(&buffer);
        start = end;

        end = start + 4;
        let card_id = u32::from_le_bytes(data[start..end].try_into().unwrap()); // 4 bytes
        start = end;

        end = start + 4;
        let required_time = u32::from_le_bytes(data[start..end].try_into().unwrap()); // 4 bytes
        start = end;

        end = start + 1;
        let missed = data[start]; // 1 bytes
        // start = end;

        let mob_action = MobCommand::AttackFromMobToHero(MobToHeroData
        {
            hero_id: player_id,
            card_id,
            time: required_time,
            missed,
            attacker_mob_id,
            attacker_mob_tile_id: tile_id,
        });

        cli_log::info!("got a {:?}", mob_action);

        channel_mob_tx.send(mob_action).await.unwrap();
}