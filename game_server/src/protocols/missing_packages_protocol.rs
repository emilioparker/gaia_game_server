
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

pub fn process_request(
    _player_id: u16,
    data : &[u8; 508],
    missing_packages : Arc<HashMap<u16, [AtomicU64;10]>>)
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

    // cli_log::info!("set missing packages for character {player_id}");
    start = end;
    if let Some(group) = missing_packages.get(&player_id)
    {
        for index in 0..10 
        { 
            end = start + 8;
            let missing_packet = u64::from_le_bytes(data[start..end].try_into().unwrap());
            start = end;
            // cli_log::info!("set missing {index} packet {missing_packet}");
            group[index].store(missing_packet, std::sync::atomic::Ordering::Relaxed);
        }
    }
}