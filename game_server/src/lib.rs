use std::sync::atomic::{AtomicU32, AtomicUsize};

pub mod protocols;
pub mod gameplay_service;
pub mod player;
pub mod map;
pub mod web_service;
pub mod real_time_service;
pub mod long_term_storage_service;

#[derive(Debug)]
pub struct ServerState {
    pub tx_mc_client_gameplay: AtomicUsize,
    pub tx_pc_client_gameplay: AtomicUsize,
    pub tx_bytes_gameplay_socket: AtomicUsize,
    pub tx_me_gameplay_longterm:AtomicUsize,
    pub tx_pe_gameplay_longterm:AtomicUsize
}