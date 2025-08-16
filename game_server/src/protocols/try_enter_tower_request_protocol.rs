
use std::sync::Arc;

use bytes::Bytes;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::hero::hero_card_inventory::CardItem;
use crate::hero::hero_entity::{HeroEntity, TRYING_TO_ENTER_TOWER_FLAG};
use crate::hero::hero_inventory::InventoryItem;
use crate::hero::hero_weapon_inventory::WeaponItem;
use crate::gaia_mpsc::GaiaSender;
use crate::gameplay_service::generic_command::GenericCommand;
use crate::map::GameMap;
use crate::hero::hero_command::{HeroCommand, HeroMovement};
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub async fn process_request(
    player_address : std::net::SocketAddr, 
    is_udp : bool,
    generic_channel_tx : &GaiaSender<GenericCommand>,
    data : &[u8],
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

    let mut player_entities = map.character.lock().await;
    let player_option = player_entities.get_mut(&player_id);

    let result = if let Some(player_entity) = player_option 
    {
        player_entity.set_flag(TRYING_TO_ENTER_TOWER_FLAG, true);
        player_entity.version += 1;
        cli_log::info!(" player entity set {}", player_entity.flags);
        let data = pack_hero_data(player_entity);
        Some(data)
    }
    else 
    {
        cli_log::error!("Try Enter tower request - player not found {}" , player_id);
        None
    };

    drop(player_entities); // we drop the lock asap, we can do what we want later.

    if let Some(data) = result 
    {
        generic_channel_tx.send(GenericCommand{player_address, is_udp, data : Bytes::from(data.to_vec())}).await.unwrap();
    }
}

pub fn pack_hero_data(hero_data: &HeroEntity)
    -> Vec<u8>
{
    let mut encoder = ZlibEncoder::new(Vec::new(),Compression::new(9));        
    // we write the protocol
    let hero_update_response = crate::protocols::Protocol::HeroData as u8;
    let buffer = [hero_update_response;1];
    std::io::Write::write_all(&mut encoder, &buffer).unwrap();
    let data = hero_data.to_bytes();
    std::io::Write::write_all(&mut encoder, &data).unwrap();

    let compressed_bytes = encoder.reset(Vec::new()).unwrap();
    compressed_bytes
}