
use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::gameplay_service::generic_command::GenericCommand;
use crate::map::GameMap;
use crate::character::character_command::{CharacterCommand, CharacterMovement};
use crate::protocols::inventory_request_protocol::pack_inventory;
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub async fn process_request(
    _player_id: u16,
    player_address : std::net::SocketAddr, 
    generic_channel_tx : &Sender<GenericCommand>,
    data : &[u8; 508],
    map : Arc<GameMap>)
{
    cli_log::info!("---- card crafting request");
    let start = 1;
    let end = start + 8;
    let _player_session_id = u64::from_le_bytes(data[start..end].try_into().unwrap());

    let start = end;
    let end = start + 2;
    let player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());

    let start = end;
    let end = start + 1;
    let _faction = data[start];

    let mut player_entities = map.character.lock().await;
    let player_option = player_entities.get_mut(&player_id);

    if let Some(player_entity) = player_option 
    {
        let result = player_entity.craft_card(&map.definitions);

        if result
        { 
            let inventory = player_entity.inventory.clone();
            let card_inventory = player_entity.card_inventory.clone();
            let weapon_inventory = player_entity.weapon_inventory.clone();
            let version = player_entity.inventory_version;
            drop(player_entities); // we drop the lock asap, we can do what we want later.

            let compressed_bytes = pack_inventory(inventory, card_inventory, weapon_inventory, version);
            generic_channel_tx.send(GenericCommand{player_address, data : compressed_bytes}).await.unwrap();
        }
    }
    else 
    {
        cli_log::info!("Inventory Request - player not found {}" , player_id);
    };
}