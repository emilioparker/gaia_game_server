
use std::sync::Arc;

use bytes::Bytes;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::hero::hero_card_inventory::CardItem;
use crate::hero::hero_entity::{HeroEntity, INSIDE_TOWER_FLAG};
use crate::hero::hero_inventory::InventoryItem;
use crate::hero::hero_weapon_inventory::WeaponItem;
use crate::gaia_mpsc::GaiaSender;
use crate::gameplay_service::generic_command::GenericCommand;
use crate::map::tetrahedron_id::TetrahedronId;
use crate::map::GameMap;
use crate::hero::hero_command::{HeroCommand, HeroMovement};
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub async fn process_request(
    player_address : std::net::SocketAddr, 
    hero_channel_tx : &GaiaSender<HeroCommand>,
    data : &[u8],
    map : &Arc<GameMap>)
{
    cli_log::info!("---- enter or exit tower");
    let start = 1;
    let end = start + 8;
    let _player_session_id = u64::from_le_bytes(data[start..end].try_into().unwrap());
    let start = end;

    let end = start + 2;
    let player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());
    let start = end;


    let end = start + 6;
    let mut buffer = [0u8;6];
    buffer.copy_from_slice(&data[start..end]);
    let tile_id = TetrahedronId::from_bytes(&buffer);
    let start = end;

    let end = start + 1;
    let faction = data[start];
    let start = end;

    // let mut player_entities = map.character.lock().await;
    // let player_option = player_entities.get_mut(&player_id);

    // if let Some(player_entity) = player_option 
    // {
    //     player_entity.set_flag(INSIDE_TOWER, true);
    // }
    // else 
    // {
    //     cli_log::error!("Enter tower request - player not found {}" , player_id);
    // };

    // drop(player_entities); // we drop the lock asap, we can do what we want later.

    hero_channel_tx.send(HeroCommand 
        {
            player_id,
            info: crate::hero::hero_command::HeroCommandInfo::EnterTower(tile_id, faction) 
        }).await.unwrap();
}