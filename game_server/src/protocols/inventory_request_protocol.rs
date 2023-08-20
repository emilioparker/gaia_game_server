
use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::map::GameMap;
use crate::character::character_command::CharacterCommand;
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub async fn process_request(
    player_id: u16,
    socket:&UdpSocket,
    data : &[u8; 508],
    map : Arc<GameMap>,
    _channel_tx : &Sender<CharacterCommand>)
{
    let mut start = 1;
    let mut end = start + 2;


//TODO: WE USE THIS ONE BECUASE THE OTHER ID IS 0 THE FIRST TIME... NEED TO DEBUG...
    let requested_player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());
    start = end;

    end = start + 8;
    let _session_id = u64::from_le_bytes(data[start..end].try_into().unwrap());
    start = end;

    //end = start + 1;
    let _page = data[start];
    //start = end;

    let player_entities = map.players.lock().await;
    let player_option = player_entities.get(&requested_player_id);

    let (inventory, hash) = if let Some(player_entity) = player_option {
        (player_entity.inventory.clone(), player_entity.inventory_hash)
    }
    else {
        println!("Inventory Request - player not found {}" , player_id);
        (Vec::new(), 1)
    };

    drop(player_entities); // we drop the lock asap, we can do what we want later.
    let mut encoder = ZlibEncoder::new(Vec::new(),Compression::new(9));        

    // we write the protocol
    let buffer = [5u8;1];
    std::io::Write::write_all(&mut encoder, &buffer).unwrap();
    // we write the amount of items.
    let item_len_bytes = u32::to_le_bytes(inventory.len() as u32);
    std::io::Write::write_all(&mut encoder, &item_len_bytes).unwrap();

    let hash_bytes = u32::to_le_bytes(hash);
    std::io::Write::write_all(&mut encoder, &hash_bytes).unwrap();

    let mut offset = 0;
    for item in inventory {
        let buffer = item.to_bytes();
        offset += buffer.len();
        std::io::Write::write_all(&mut encoder, &buffer).unwrap();
    }

    let compressed_bytes = encoder.reset(Vec::new()).unwrap();
    let _len = socket.send(&compressed_bytes).await.unwrap();
    println!("Inventory - {:?} bytes sent, original {:?}", _len, offset);
}