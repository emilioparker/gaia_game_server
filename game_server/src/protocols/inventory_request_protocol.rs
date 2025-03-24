
use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::character::character_card_inventory::CardItem;
use crate::character::character_inventory::InventoryItem;
use crate::character::character_weapon_inventory::WeaponItem;
use crate::gaia_mpsc::GaiaSender;
use crate::gameplay_service::generic_command::GenericCommand;
use crate::map::GameMap;
use crate::character::character_command::{CharacterCommand, CharacterMovement};
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub async fn process_request(
    _player_id: u16,
    player_address : std::net::SocketAddr, 
    generic_channel_tx : &GaiaSender<GenericCommand>,
    data : &[u8; 508],
    map : &Arc<GameMap>)
{
    cli_log::info!("---- inventory request");
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

    let (inventory, card_inventory, weapon_inventory, inventory_version) = if let Some(player_entity) = player_option 
    {
        (player_entity.inventory.clone(), player_entity.card_inventory.clone(), player_entity.weapon_inventory.clone(), player_entity.inventory_version)
    }
    else {
        cli_log::info!("Inventory Request - player not found {}" , player_id);
        (Vec::new(), Vec::new(), Vec::new(), 1)
    };

    drop(player_entities); // we drop the lock asap, we can do what we want later.

    // we pay the price of cloning, but just because compressing might be costly.
    let compressed_bytes = pack_inventory(inventory, card_inventory, weapon_inventory, inventory_version);
    generic_channel_tx.send(GenericCommand{player_address, data : compressed_bytes}).await.unwrap();
}

pub fn pack_inventory(
    inventory: Vec<InventoryItem>, 
    card_inventory: Vec<CardItem>,
    weapon_inventory : Vec<WeaponItem>,
    inventory_version: u8)
    -> Vec<u8>
{
    let mut encoder = ZlibEncoder::new(Vec::new(),Compression::new(9));        
    // we write the protocol
    let inventory_request = crate::protocols::Protocol::InventoryRequest as u8;
    let buffer = [inventory_request;1];
    std::io::Write::write_all(&mut encoder, &buffer).unwrap();
    // we write the amount of items.
    let item_len_bytes = u32::to_le_bytes(inventory.len() as u32);
    std::io::Write::write_all(&mut encoder, &item_len_bytes).unwrap();

    // cli_log::info!("--- inventory length {}", inventory.len());

    for item in inventory 
    {
        // cli_log::info!("---- item {:?}", item);
        let buffer = item.to_bytes();
        std::io::Write::write_all(&mut encoder, &buffer).unwrap();
    }

    // card inventory
    let card_inventory_len_bytes = u32::to_le_bytes(card_inventory.len() as u32);
    std::io::Write::write_all(&mut encoder, &card_inventory_len_bytes).unwrap();

    // cli_log::info!("--- inventory length {}", card_inventory.len());
    for item in card_inventory 
    {
        // cli_log::info!("---- card {:?}", item);
        let buffer = item.to_bytes();
        std::io::Write::write_all(&mut encoder, &buffer).unwrap();
    }

    // weapon inventory
    let weapon_inventory_len_bytes = u32::to_le_bytes(weapon_inventory.len() as u32);
    std::io::Write::write_all(&mut encoder, &weapon_inventory_len_bytes).unwrap();

    // cli_log::info!("--- weapon inventory length {}", weapon_inventory.len());
    for item in weapon_inventory 
    {
        // cli_log::info!("---- card {:?}", item);
        let buffer = item.to_bytes();
        std::io::Write::write_all(&mut encoder, &buffer).unwrap();
    }

    std::io::Write::write_all(&mut encoder, &[inventory_version]).unwrap();
    let compressed_bytes = encoder.reset(Vec::new()).unwrap();
    compressed_bytes
}