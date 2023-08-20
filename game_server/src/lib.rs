use std::sync::atomic::{AtomicUsize};

pub mod protocols;
pub mod gameplay_service;
pub mod character;
pub mod map;
pub mod web_service;
pub mod real_time_service;
pub mod long_term_storage_service;
pub mod tower;

#[derive(Debug)]
pub struct ServerState {
    pub tx_mc_client_gameplay: AtomicUsize,
    pub tx_pc_client_gameplay: AtomicUsize,
    pub tx_tc_client_gameplay: AtomicUsize,
    pub tx_bytes_gameplay_socket: AtomicUsize,
    pub tx_me_gameplay_longterm:AtomicUsize,
    pub tx_me_gameplay_webservice:AtomicUsize,
    pub tx_pe_gameplay_longterm:AtomicUsize
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