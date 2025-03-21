use std::collections::HashMap;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use map::GameMap;
use strum::IntoEnumIterator;
use strum_macros::Display;
use strum_macros::EnumIter;
use strum_macros::EnumString;

pub mod protocols;
pub mod gameplay_service;
pub mod character;
pub mod map;
pub mod web_service;
pub mod clients_service;
pub mod long_term_storage_service;
pub mod tower;
pub mod chat;
pub mod chat_service;
pub mod definitions;
pub mod mob;
pub mod buffs;
pub mod ability_user;
pub mod app;
pub mod gaia_mpsc;

pub struct AppData
{
    pub game_data: Arc<GameMap>,
    pub game_status: Arc<ServerState>
}

pub const SERVER_STATE_SIZE: usize = 20;

#[derive(PartialEq, Eq, Hash, Clone, EnumIter, EnumString, Display)]
pub enum ServerChannels 
{
    TX_GC_ClIENTS_GAMEPLAY,
    TX_MC_CLIENTS_GAMEPLAY,
    TX_PC_CLIENTS_GAMEPLAY,
    TX_TC_CLIENTS_GAMEPLAY,
    TX_CC_CLIENTS_GAMEPLAY,
    TX_MOC_CLIENTS_GAMEPLAY,
    TX_MOE_GAMEPLAY_WEBSERVICE,
    TX_PACKETS_GAMEPLAY_CHAT_CLIENTS,
    TX_MC_WEBSERVICE_GAMEPLAY,
    TX_ME_GAMEPLAY_LONGTERM,
    TX_ME_GAMEPLAY_WEBSERVICE,
    TX_PE_GAMEPLAY_LONGTERM,
    TX_TE_GAMEPLAY_LONGTERM,
    TX_TE_GAMEPLAY_WEBSERVICE,
    TX_CE_CHAT_WEBSERVICE,
    TX_SAVED_LONGTERM_WEBSERVICE,
    TX_TE_SAVED_LONGTERM_WEBSERVICE,
}

pub struct ServerState 
{
    pub channels : HashMap<ServerChannels, AtomicU16>,
    pub received_packets:AtomicU64,
    pub received_bytes:AtomicU64,
    pub online_players: AtomicU32,
    pub sent_udp_packets:AtomicU64,
    pub sent_game_packets:AtomicU64,
    pub sent_bytes:AtomicU64,
    pub total_players:AtomicU32,
}

impl ServerState 
{
    pub fn get_stats(&self) -> [u16; 10]
    {
        let order = std::sync::atomic::Ordering::Relaxed;
        let mut stats :[u16; 10] = [0;10];

        for (i, channel) in ServerChannels::iter().enumerate()
        {
            if i >= 10
            {
                break;
            }

            let capacity = self.channels[&channel].load(order);
            stats[i] = capacity;
        }

        stats
    }

    pub fn stats_to_bytes(stats : &[u16; 10]) -> [u8; SERVER_STATE_SIZE]
    {
        let mut buffer = [0u8; SERVER_STATE_SIZE];
        let mut offset = 0;

        for stat in stats
        {
            let tx_mc_client_gameplay_stat = u16::to_le_bytes(*stat); // 2 bytes
            let end = offset + 2; 
            buffer[offset..end].copy_from_slice(&tx_mc_client_gameplay_stat);
            offset = end;
        }
        buffer
    }

    pub fn get_size() -> usize 
    {
        SERVER_STATE_SIZE
    }
}

pub fn get_faction_code(faction : &str) -> u8
{
    match faction {
        "none" => 0,
        "red" => 1,
        "green" => 2,
        "blue" => 3,
        "corruption" => 4,
        _ => 255
    }
}

pub fn get_faction_from_code(faction : u8) -> String 
{
    match faction {
        0 => "none".to_owned(),
        1 => "red".to_owned(),
        2 => "green".to_owned(),
        3 => "blue".to_owned(),
        4 => "corruption".to_owned(),
        _ => "none".to_owned()
    }
}