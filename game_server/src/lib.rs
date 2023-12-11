use std::sync::atomic::AtomicI32;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicU16;

pub mod protocols;
pub mod gameplay_service;
pub mod character;
pub mod map;
pub mod web_service;
pub mod real_time_service;
pub mod long_term_storage_service;
pub mod tower;
pub mod chat;
pub mod chat_service;

pub const SERVER_STATE_SIZE: usize = 20;

#[derive(Debug)]
pub struct ServerState 
{
    pub tx_mc_client_gameplay: AtomicU16,
    pub tx_pc_client_gameplay: AtomicU16,
    pub tx_tc_client_gameplay: AtomicU16,
    pub tx_cc_client_gameplay: AtomicU16,
    pub tx_bytes_gameplay_socket: AtomicU16,
    pub tx_me_gameplay_longterm:AtomicU16,
    pub tx_me_gameplay_webservice:AtomicU16,
    pub tx_pe_gameplay_longterm:AtomicU16,
    pub online_players: AtomicI32,
    pub total_players:AtomicU32,
}

impl ServerState 
{
    pub fn get_stats(&self) -> [u16; 10]
    {
        let order = std::sync::atomic::Ordering::Relaxed;
        let stats :[u16; 10] = 
        [
            self.tx_mc_client_gameplay.load(order),
            self.tx_pc_client_gameplay.load(order),
            self.tx_tc_client_gameplay.load(order),
            self.tx_cc_client_gameplay.load(order),
            self.tx_bytes_gameplay_socket.load(order),
            self.tx_me_gameplay_longterm.load(order),
            self.tx_me_gameplay_webservice.load(order),
            self.tx_pe_gameplay_longterm.load(order),
            self.online_players.load(order) as f32 as u16,
            self.total_players.load(order) as f32 as u16
        ];

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