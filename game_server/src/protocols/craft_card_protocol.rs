
use std::sync::Arc;

use bytes::Bytes;
use crate::gaia_mpsc::GaiaSender;
use crate::gameplay_service::generic_command::GenericCommand;
use crate::map::GameMap;
use crate::hero::hero_command::{HeroCommand, HeroMovement};
use crate::protocols::inventory_request_protocol::pack_inventory;

pub async fn process_request(
    player_address : std::net::SocketAddr, 
    is_udp: bool,
    tx_gc_clients_gameplay : &GaiaSender<GenericCommand>,
    data : &[u8],
    map : &Arc<GameMap>)
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
            tx_gc_clients_gameplay.send(GenericCommand{player_address, is_udp, data : Bytes::from(compressed_bytes)}).await.unwrap();
        }
    }
    else 
    {
        cli_log::info!("Inventory Request - player not found {}" , player_id);
    };
}