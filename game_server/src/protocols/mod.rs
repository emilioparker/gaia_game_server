pub mod ping_protocol;
pub mod movement_protocol;
pub mod resource_extraction_protocol;
pub mod inventory_request_protocol;
pub mod layfoundation_protocol;
pub mod lay_wall_foundation_protocol;
pub mod build_protocol;
pub mod mob_attacks_character_protocol;
pub mod spawn_mob_protocol;
pub mod mob_moves_protocol;
pub mod claim_mob_ownership;
pub mod cast_mob_from_character_protocol;
pub mod missing_packages_protocol;
pub mod attack_tower_protocol;
pub mod repair_tower_protocol;
pub mod chat_message_protocol;
pub mod sell_item_protocol;
pub mod buy_item_protocol;
pub mod use_item_protocol;
pub mod equip_item_protocol;
pub mod respawn_protocol;
pub mod action_protocol;
pub mod greet_protocol;
pub mod activate_buff_protocol;
pub mod character_attacks_character_protocol;
pub mod disconnect_protocol;
pub mod touch_mob_protocol;
pub mod cast_mob_from_mob_protocol;
pub mod craft_card_protocol;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU16;

use crate::gaia_mpsc::GaiaSender;
use crate::gameplay_service::generic_command::GenericCommand;
use crate::kingdom::KingdomCommand;
use crate::mob::mob_command::MobCommand;
use crate::ServerState;
use crate::chat::ChatCommand;
use crate::map::GameMap;
use crate::map::map_entity::MapCommand;
use crate::hero::hero_command::HeroCommand;
use crate::tower::TowerCommand;


pub enum Protocol
{
    Ping = 1,
    CharacterMovement = 2,
    GlobalState = 3,
    ResourceExtraction = 4,
    InventoryRequest = 5,
    LayFoundation = 6,
    Build = 7,
    MobAttacksWalker = 8,
    SpawnMob = 9,
    MobMoves = 10,
    ControlMob = 11,
    AttackMob = 12,
    MissingPackets = 13,
    AttackTower = 14,
    RepairTower = 15,
    ChatMessage = 16,
    BuildWall = 17,
    SellItem = 18,
    BuyItem = 19,
    UseItem = 20,
    EquipItem = 21,
    Respawn = 22,
    CharacterAction = 23,
    Greet = 24,
    ActivateBuff = 25,
    CharacterAttacksCharacter = 26,
    AttackStructure = 27,
    TouchMob = 28,
    CastMobFromMob = 29,
    CraftCard = 30,
}
    
pub async fn route_packet(
    player_address : std::net::SocketAddr, 
    // socket: &UdpSocket,
    data : &[u8],
    packet_size: usize,
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    regions: &Arc<HashMap<u16, [AtomicU16;3]>>,
    tx_gc_clients_gameplay: &GaiaSender<GenericCommand>,
    tx_pc_clients_gameplay: &GaiaSender<HeroCommand>,
    tx_mc_clients_gameplay: &GaiaSender<MapCommand>,
    tx_moc_clients_gameplay: &GaiaSender<MobCommand>,
    tx_tc_clients_gameplay: &GaiaSender<TowerCommand>,
    tx_kc_clients_gameplay: &GaiaSender<KingdomCommand>,
    tx_cc_clients_gameplay: &GaiaSender<ChatCommand>
){

    server_state.received_packets.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    server_state.received_bytes.fetch_add(packet_size as u64, std::sync::atomic::Ordering::Relaxed);

    match data.get(0) 
    {
        Some(protocol) if *protocol == Protocol::Ping as u8 => 
        {
            ping_protocol::process_ping(player_address, tx_gc_clients_gameplay, data).await;
        },
        Some(protocol) if *protocol == Protocol::SellItem as u8 => 
        {
            sell_item_protocol::process(data, tx_pc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::BuyItem as u8 => 
        {
            buy_item_protocol::process(data, tx_pc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::UseItem as u8 => 
        {
            use_item_protocol::process(data, tx_pc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::EquipItem as u8 => 
        {
            equip_item_protocol::process(data, tx_pc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::InventoryRequest as u8 => 
        {
            inventory_request_protocol::process_request(player_address, tx_gc_clients_gameplay, data, map).await;
        },
        Some(protocol) if *protocol == Protocol::LayFoundation as u8 => 
        {
            layfoundation_protocol::process_construction(data, tx_mc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::CharacterMovement as u8 => 
        {
            movement_protocol::process_movement(data, regions, tx_pc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::ResourceExtraction as u8 => 
        {
            resource_extraction_protocol::process(data, tx_mc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::Build as u8 => 
        {
            build_protocol::process(data, tx_mc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::MobAttacksWalker as u8 => 
        { // used by mobs and towers.
            mob_attacks_character_protocol::process(data, tx_moc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::SpawnMob as u8 => 
        {
            spawn_mob_protocol::process(data, tx_moc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::MobMoves as u8 => 
        {
            mob_moves_protocol::process(data, tx_moc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::ControlMob as u8 => 
        {
            claim_mob_ownership::process(data, tx_moc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::AttackMob as u8 => 
        {
            cast_mob_from_character_protocol::process(data, tx_moc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::MissingPackets as u8 => 
        {
            // let capacity = tx_mc_clients_gameplay.capacity();
            // server_state.tx_mc_clients_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            // missing_packages_protocol::process_request(player_id, data, missing_packets);
        },
        Some(protocol) if *protocol == Protocol::AttackTower as u8 => 
        {
            attack_tower_protocol::process(data, tx_tc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::RepairTower as u8 => 
        {
            repair_tower_protocol::process(data, tx_tc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::RepairTower as u8 => 
        {
            repair_tower_protocol::process(data, tx_tc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::ChatMessage as u8 => 
        {
            chat_message_protocol::process(data, tx_cc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::BuildWall as u8 => 
        {
            lay_wall_foundation_protocol::process_construction(data, tx_mc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::Respawn as u8 => 
        {
            cli_log::info!("--------------------- process respawn");
            respawn_protocol::process_respawn(data, regions, tx_pc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::CharacterAction as u8 => 
        {
            cli_log::info!("--------------------- process character action");
            action_protocol::process(data, tx_pc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::Greet as u8 => 
        {
            cli_log::info!("--------------------- process greet");
            greet_protocol::process(data, tx_pc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::ActivateBuff as u8 => 
        {
            cli_log::info!("--------------------- process buff");
            activate_buff_protocol::process(data, tx_pc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::CharacterAttacksCharacter as u8 => 
        {
            cli_log::info!("--------------------- process character attack");
            character_attacks_character_protocol::process(data, tx_pc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::TouchMob as u8 => 
        {
            cli_log::info!("--------------------- process touch mob");
            touch_mob_protocol::process(data, tx_moc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::CastMobFromMob as u8 => 
        {
            cli_log::info!("--------------------- process touch mob");
            cast_mob_from_mob_protocol::process(data, tx_moc_clients_gameplay).await;
        },
        Some(protocol) if *protocol == Protocol::CraftCard as u8 => 
        {
            cli_log::info!("--------------------- process craft card");
            craft_card_protocol::process_request(player_address, tx_gc_clients_gameplay, data, map).await;
        },
        unknown_protocol => 
        {
            cli_log::error!("unknown protocol {:?}", unknown_protocol);
        }
    }
}
