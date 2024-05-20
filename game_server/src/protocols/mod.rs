pub mod ping_protocol;
pub mod movement_protocol;
pub mod resource_extraction_protocol;
pub mod inventory_request_protocol;
pub mod layfoundation_protocol;
pub mod lay_wall_foundation_protocol;
pub mod build_protocol;
pub mod tile_attacks_walker_protocol;
pub mod spawn_mob_protocol;
pub mod mob_moves_protocol;
pub mod claim_mob_ownership;
pub mod attack_mob_protocol;
pub mod missing_packages_protocol;
pub mod attack_tower_protocol;
pub mod repair_tower_protocol;
pub mod chat_message_protocol;
pub mod sell_item_protocol;
pub mod buy_item_protocol;
pub mod use_item_protocol;
pub mod battle_protocols;
pub mod equip_item_protocol;
pub mod respawn_protocol;
pub mod action_protocol;
pub mod greet_protocol;
pub mod activate_buff_protocol;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

use crate::battle::battle_command::BattleCommand;
use crate::gameplay_service::player_commands_processor::activate_buff;
use crate::ServerState;
use crate::chat::ChatCommand;
use crate::map::GameMap;
use crate::map::map_entity::MapCommand;
use crate::character::character_command::CharacterCommand;
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
    TileAttacksWalker = 8,
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
}
    
pub async fn route_packet(
    player_id: u16,
    socket: &UdpSocket,
    data : &[u8; 508],
    map : Arc<GameMap>,
    server_state: &Arc<ServerState>,
    missing_packets : Arc<HashMap<u16, [AtomicU64;10]>>,
    channel_tx : &Sender<CharacterCommand>,
    channel_map_tx : &Sender<MapCommand>,
    channel_battle_tx : &Sender<BattleCommand>,
    channel_tower_tx : &Sender<TowerCommand>,
    channel_chat_tx : &Sender<ChatCommand>
){

    match data.get(0) {
        Some(protocol) if *protocol == Protocol::Ping as u8 => {
            ping_protocol::process_ping(socket, data).await;
        },
        Some(protocol) if *protocol == Protocol::SellItem as u8 => {
            let capacity = channel_tx.capacity();
            server_state.tx_pc_client_gameplay.store( capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            sell_item_protocol::process(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::BuyItem as u8 => {
            let capacity = channel_tx.capacity();
            server_state.tx_pc_client_gameplay.store( capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            buy_item_protocol::process(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::UseItem as u8 => {
            let capacity = channel_tx.capacity();
            server_state.tx_pc_client_gameplay.store( capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            use_item_protocol::process(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::EquipItem as u8 => {
            let capacity = channel_battle_tx.capacity();
            server_state.tx_bc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            equip_item_protocol::process(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::InventoryRequest as u8 => {
            inventory_request_protocol::process_request(player_id, socket, data, map).await;
        },
        Some(protocol) if *protocol == Protocol::LayFoundation as u8 => {
            layfoundation_protocol::process_construction(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::CharacterMovement as u8 => {
            let capacity = channel_tx.capacity();
            server_state.tx_pc_client_gameplay.store( capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            movement_protocol::process_movement(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::ResourceExtraction as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            resource_extraction_protocol::process(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::Build as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            build_protocol::process(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::TileAttacksWalker as u8 => { // used by mobs and towers.
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            tile_attacks_walker_protocol::process(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::SpawnMob as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            spawn_mob_protocol::process(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::MobMoves as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            mob_moves_protocol::process(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::ControlMob as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            claim_mob_ownership::process(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::AttackMob as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            attack_mob_protocol::process(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::MissingPackets as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            missing_packages_protocol::process_request(player_id, data, missing_packets);
        },
        Some(protocol) if *protocol == Protocol::AttackTower as u8 => {
            let capacity = channel_tower_tx.capacity();
            server_state.tx_tc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            attack_tower_protocol::process(socket, data, channel_tower_tx).await;
        },
        Some(protocol) if *protocol == Protocol::RepairTower as u8 => {
            let capacity = channel_tower_tx.capacity();
            server_state.tx_tc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            repair_tower_protocol::process(socket, data, channel_tower_tx).await;
        },
        Some(protocol) if *protocol == Protocol::RepairTower as u8 => {
            let capacity = channel_tower_tx.capacity();
            server_state.tx_tc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            repair_tower_protocol::process(socket, data, channel_tower_tx).await;
        },
        Some(protocol) if *protocol == Protocol::ChatMessage as u8 => {
            let capacity = channel_chat_tx.capacity();
            server_state.tx_cc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            chat_message_protocol::process(socket, data, channel_chat_tx).await;
        },
        Some(protocol) if *protocol == Protocol::BuildWall as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            lay_wall_foundation_protocol::process_construction(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::Respawn as u8 => {
            println!("--------------------- process respawn");
            let capacity = channel_battle_tx.capacity();
            server_state.tx_pc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            respawn_protocol::process_respawn(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::CharacterAction as u8 => {
            println!("--------------------- process character action");
            let capacity = channel_battle_tx.capacity();
            server_state.tx_pc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            action_protocol::process(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::Greet as u8 => 
        {
            println!("--------------------- process greet");
            let capacity = channel_battle_tx.capacity();
            server_state.tx_pc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            greet_protocol::process(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::ActivateBuff as u8 => 
        {
            println!("--------------------- process buff");
            let capacity = channel_battle_tx.capacity();
            server_state.tx_pc_client_gameplay.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
            activate_buff_protocol::process(socket, data, channel_tx).await;
        },
        unknown_protocol => {
            println!("unknown protocol {:?}", unknown_protocol);
        }
    }
}
