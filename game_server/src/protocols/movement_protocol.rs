use std::{collections::HashMap, sync::{atomic::AtomicU16, Arc}};

use tokio::{sync::mpsc::Sender, net::UdpSocket};

use crate::{hero::hero_command::{HeroCommand, HeroCommandInfo, HeroMovement}, gaia_mpsc::GaiaSender, map::tetrahedron_id::TetrahedronId};


pub async fn process_movement(
    data : &[u8],
    regions : &Arc<HashMap<u16, [AtomicU16;3]>>,
    channel_tx : &GaiaSender<HeroCommand>)
{
    //1 - protocolo 1 bytes
    //2 - id 8 bytes
    // the rest depends on the code.
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

    end = start + 2;
    let region_1 = u16::from_le_bytes(data[start..end].try_into().unwrap());
    start = end;

    end = start + 2;
    let region_2 = u16::from_le_bytes(data[start..end].try_into().unwrap());
    start = end;

    end = start + 2;
    let region_3 = u16::from_le_bytes(data[start..end].try_into().unwrap());
    start = end;

    end = start + 6;
    let mut buffer = [0u8;6];
    buffer.copy_from_slice(&data[start..end]);
    let position_tile_id = TetrahedronId::from_bytes(&buffer);
    start = end;

    end = start + 6;
    let mut buffer = [0u8;6];
    buffer.copy_from_slice(&data[start..end]);
    let second_position_tile_id = TetrahedronId::from_bytes(&buffer);
    start = end;

    end = start + 4;
    let vertex_id = i32::from_le_bytes(data[start..end].try_into().unwrap());
    start = end;

    let mut path : [u8;6] = [0,0,0,0,0,0];
    for i in 0..6
    {
        end = start + 1;
        path[i] = data[start];
        start = end;
    }

    end = start + 4;
    let start_time = u32::from_le_bytes(data[start..end].try_into().unwrap());
    start = end;

    // end = start + 1;
    let dash = data[start];
    // start = end;

    let player_regions = regions.get(&player_id).unwrap();
    player_regions[0].store(region_1, std::sync::atomic::Ordering::Relaxed);
    player_regions[1].store(region_2, std::sync::atomic::Ordering::Relaxed);
    player_regions[2].store(region_3, std::sync::atomic::Ordering::Relaxed);

    cli_log::info!("regions: {} {} {}", region_1, region_2, region_3);

    let action = HeroMovement 
    {
        player_id,
        position: position_tile_id,
        second_position: second_position_tile_id,
        vertex_id,
        path,
        time:start_time,
        dash: dash == 1
    };

    let character_command = HeroCommand
    {
        player_id,
        info: HeroCommandInfo::Movement(action)
    };



    channel_tx.send(character_command).await.unwrap();
}