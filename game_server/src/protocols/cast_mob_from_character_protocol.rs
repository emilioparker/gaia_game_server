
use crate::gaia_mpsc::GaiaSender;
use crate::map::tetrahedron_id::TetrahedronId;
use crate::mob::mob_command::HeroToMobData;
use crate::mob::mob_command::MobCommand;


pub async fn process(data : &[u8],  channel_mob_tx : &GaiaSender<MobCommand>)
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

    end = start + 4;
    let mob_id = u32::from_le_bytes(data[start..end].try_into().unwrap()); // 4 bytes
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

    // end = start + 1;
    // let active_effect = data[start]; // 1 bytes
    // start = end;

    end = start + 1;
    let missed = data[start]; // 1 bytes
    start = end;

    // cli_log::info!("active effect {active_effect}");

    let mob_action = MobCommand::CastFromHeroToMob(HeroToMobData
    {
        hero_id: player_id,
        card_id,
        time: required_time,
        missed,
        target_mob_id: mob_id,
        target_mob_tile_id: tile_id,
    });
    
    channel_mob_tx.send(mob_action).await.unwrap();
}