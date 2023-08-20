pub mod ping_protocol;
pub mod movement_protocol;
pub mod interaction_protocol;
pub mod inventory_request_protocol;
pub mod layfoundation_protocol;
pub mod build_protocol;
pub mod tile_attacks_walker_protocol;
pub mod spawn_mob_protocol;
pub mod mob_moves_protocol;
pub mod claim_mob_ownership;
pub mod attack_mob_protocol;
pub mod missing_packages_protocol;
pub mod attack_tower_protocol;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

use crate::ServerState;
use crate::map::GameMap;
use crate::map::map_entity::MapCommand;
use crate::character::character_command::CharacterCommand;
use crate::tower::TowerCommand;


pub enum Protocol
{
    Ping = 1,
    Action = 2,
    GlobalState = 3,
    Interaction = 4,
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
    channel_tower_tx : &Sender<TowerCommand>
){

    match data.get(0) {
        Some(protocol) if *protocol == Protocol::Ping as u8 => {
            ping_protocol::process_ping(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::InventoryRequest as u8 => {
            inventory_request_protocol::process_request(player_id, socket, data, map, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::LayFoundation as u8 => {
            layfoundation_protocol::process_construction(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::Action as u8 => {
            let capacity = channel_tx.capacity();
            server_state.tx_pc_client_gameplay.store(capacity, std::sync::atomic::Ordering::Relaxed);
            movement_protocol::process_movement(socket, data, channel_tx).await;
        },
        Some(protocol) if *protocol == Protocol::Interaction as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity, std::sync::atomic::Ordering::Relaxed);
            interaction_protocol::process_interaction(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::Build as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity, std::sync::atomic::Ordering::Relaxed);
            build_protocol::process(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::TileAttacksWalker as u8 => { // used by mobs and towers.
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity, std::sync::atomic::Ordering::Relaxed);
            tile_attacks_walker_protocol::process(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::SpawnMob as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity, std::sync::atomic::Ordering::Relaxed);
            spawn_mob_protocol::process(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::MobMoves as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity, std::sync::atomic::Ordering::Relaxed);
            mob_moves_protocol::process(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::ControlMob as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity, std::sync::atomic::Ordering::Relaxed);
            claim_mob_ownership::process(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::AttackMob as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity, std::sync::atomic::Ordering::Relaxed);
            attack_mob_protocol::process(socket, data, channel_map_tx).await;
        },
        Some(protocol) if *protocol == Protocol::MissingPackets as u8 => {
            let capacity = channel_map_tx.capacity();
            server_state.tx_mc_client_gameplay.store(capacity, std::sync::atomic::Ordering::Relaxed);
            missing_packages_protocol::process_request(player_id, data, missing_packets);
        },
        Some(protocol) if *protocol == Protocol::AttackTower as u8 => {
            let capacity = channel_tower_tx.capacity();
            server_state.tx_tc_client_gameplay.store(capacity, std::sync::atomic::Ordering::Relaxed);
            attack_tower_protocol::process(socket, data, channel_tower_tx).await;
        },
        unknown_protocol => {
            println!("unknown protocol {:?}", unknown_protocol);
        }
    }
}
