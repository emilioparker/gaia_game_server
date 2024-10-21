
use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::gameplay_service::generic_command::GenericCommand;
use crate::map::GameMap;
use crate::character::character_command::{CharacterCommand, CharacterMovement};
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub async fn process_request(
    _player_id: u16,
    player_address : std::net::SocketAddr, 
    generic_channel_tx : &Sender<GenericCommand>,
    data : &[u8; 508],
    map : Arc<GameMap>)
{
    let start = 1;
    let end = start + 8;
    let _player_session_id = u64::from_le_bytes(data[start..end].try_into().unwrap());

    let start = end;
    let end = start + 2;
    let player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());

    let start = end;
    let end = start + 1;
    let _faction = data[start];

    let start = end;
    let _page = data[start];

    let player_entities = map.character.lock().await;
    let player_option = player_entities.get(&player_id);

    let (inventory, card_inventory, inventory_version) = if let Some(player_entity) = player_option 
    {
        (player_entity.inventory.clone(), player_entity.card_inventory.clone(), player_entity.inventory_version)
    }
    else {
        println!("Inventory Request - player not found {}" , player_id);
        (Vec::new(), Vec::new(), 1)
    };

    drop(player_entities); // we drop the lock asap, we can do what we want later.
    let mut encoder = ZlibEncoder::new(Vec::new(),Compression::new(9));        

    // we write the protocol
    let buffer = [5u8;1];
    std::io::Write::write_all(&mut encoder, &buffer).unwrap();
    // we write the amount of items.
    let item_len_bytes = u32::to_le_bytes(inventory.len() as u32);
    std::io::Write::write_all(&mut encoder, &item_len_bytes).unwrap();

    for item in inventory 
    {
        let buffer = item.to_bytes();
        std::io::Write::write_all(&mut encoder, &buffer).unwrap();
    }

    // card inventory
    let card_inventory_len_bytes = u32::to_le_bytes(card_inventory.len() as u32);
    std::io::Write::write_all(&mut encoder, &card_inventory_len_bytes).unwrap();

    for item in card_inventory 
    {
        let buffer = item.to_bytes();
        std::io::Write::write_all(&mut encoder, &buffer).unwrap();
    }

    std::io::Write::write_all(&mut encoder, &[inventory_version]).unwrap();
    let compressed_bytes = encoder.reset(Vec::new()).unwrap();
    generic_channel_tx.send(GenericCommand{player_address, data : compressed_bytes}).await.unwrap();
}