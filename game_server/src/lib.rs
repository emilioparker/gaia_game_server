use std::collections::HashMap;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use map::tetrahedron_id::TetrahedronId;
use map::GameMap;
use strum::IntoEnumIterator;
use strum_macros::Display;
use strum_macros::EnumIter;
use strum_macros::EnumString;

pub mod protocols;
pub mod gameplay_service;
pub mod hero;
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
pub mod http_service;

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
    // long term data.
    pub pending_regions_to_save:AtomicU32,
    pub saved_regions:AtomicU32,
    pub last_regions_save_timestamp:AtomicU64,

    pub pending_character_entities_to_save:AtomicU32,
    pub saved_character_entities:AtomicU32,
    pub last_character_entities_save_timestamp:AtomicU64,

    pub pending_tower_entities_to_save:AtomicU32,
    pub saved_tower_entities:AtomicU32,
    pub last_tower_entities_save_timestamp:AtomicU64,
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


pub fn get_regions_by_id() -> HashMap<TetrahedronId, u16>
{
    let mut set = HashMap::new();
    let ids = get_regions_by_code();
    for (i,region) in ids.iter().enumerate()
    {
        if i == 0 
        {
            continue
        }
        set.insert(region.clone(), i as u16);
    }

    set
}

pub fn get_regions_by_code() -> [TetrahedronId; 321]
{
    let region_ids : [TetrahedronId;321] =
    [
        TetrahedronId::from_string("a00"), // the first one is global, added here just to offset the indices
        TetrahedronId::from_string("a00"),
        TetrahedronId::from_string("a01"),
        TetrahedronId::from_string("a02"),
        TetrahedronId::from_string("a03"),
        TetrahedronId::from_string("a10"),
        TetrahedronId::from_string("a11"),
        TetrahedronId::from_string("a12"),
        TetrahedronId::from_string("a13"),
        TetrahedronId::from_string("a20"),
        TetrahedronId::from_string("a21"),
        TetrahedronId::from_string("a22"),
        TetrahedronId::from_string("a23"),
        TetrahedronId::from_string("a30"),
        TetrahedronId::from_string("a31"),
        TetrahedronId::from_string("a32"),
        TetrahedronId::from_string("a33"),
        TetrahedronId::from_string("b00"),
        TetrahedronId::from_string("b01"),
        TetrahedronId::from_string("b02"),
        TetrahedronId::from_string("b03"),
        TetrahedronId::from_string("b10"),
        TetrahedronId::from_string("b11"),
        TetrahedronId::from_string("b12"),
        TetrahedronId::from_string("b13"),
        TetrahedronId::from_string("b20"),
        TetrahedronId::from_string("b21"),
        TetrahedronId::from_string("b22"),
        TetrahedronId::from_string("b23"),
        TetrahedronId::from_string("b30"),
        TetrahedronId::from_string("b31"),
        TetrahedronId::from_string("b32"),
        TetrahedronId::from_string("b33"),
        TetrahedronId::from_string("c00"),
        TetrahedronId::from_string("c01"),
        TetrahedronId::from_string("c02"),
        TetrahedronId::from_string("c03"),
        TetrahedronId::from_string("c10"),
        TetrahedronId::from_string("c11"),
        TetrahedronId::from_string("c12"),
        TetrahedronId::from_string("c13"),
        TetrahedronId::from_string("c20"),
        TetrahedronId::from_string("c21"),
        TetrahedronId::from_string("c22"),
        TetrahedronId::from_string("c23"),
        TetrahedronId::from_string("c30"),
        TetrahedronId::from_string("c31"),
        TetrahedronId::from_string("c32"),
        TetrahedronId::from_string("c33"),
        TetrahedronId::from_string("d00"),
        TetrahedronId::from_string("d01"),
        TetrahedronId::from_string("d02"),
        TetrahedronId::from_string("d03"),
        TetrahedronId::from_string("d10"),
        TetrahedronId::from_string("d11"),
        TetrahedronId::from_string("d12"),
        TetrahedronId::from_string("d13"),
        TetrahedronId::from_string("d20"),
        TetrahedronId::from_string("d21"),
        TetrahedronId::from_string("d22"),
        TetrahedronId::from_string("d23"),
        TetrahedronId::from_string("d30"),
        TetrahedronId::from_string("d31"),
        TetrahedronId::from_string("d32"),
        TetrahedronId::from_string("d33"),
        TetrahedronId::from_string("e00"),
        TetrahedronId::from_string("e01"),
        TetrahedronId::from_string("e02"),
        TetrahedronId::from_string("e03"),
        TetrahedronId::from_string("e10"),
        TetrahedronId::from_string("e11"),
        TetrahedronId::from_string("e12"),
        TetrahedronId::from_string("e13"),
        TetrahedronId::from_string("e20"),
        TetrahedronId::from_string("e21"),
        TetrahedronId::from_string("e22"),
        TetrahedronId::from_string("e23"),
        TetrahedronId::from_string("e30"),
        TetrahedronId::from_string("e31"),
        TetrahedronId::from_string("e32"),
        TetrahedronId::from_string("e33"),
        TetrahedronId::from_string("f00"),
        TetrahedronId::from_string("f01"),
        TetrahedronId::from_string("f02"),
        TetrahedronId::from_string("f03"),
        TetrahedronId::from_string("f10"),
        TetrahedronId::from_string("f11"),
        TetrahedronId::from_string("f12"),
        TetrahedronId::from_string("f13"),
        TetrahedronId::from_string("f20"),
        TetrahedronId::from_string("f21"),
        TetrahedronId::from_string("f22"),
        TetrahedronId::from_string("f23"),
        TetrahedronId::from_string("f30"),
        TetrahedronId::from_string("f31"),
        TetrahedronId::from_string("f32"),
        TetrahedronId::from_string("f33"),
        TetrahedronId::from_string("g00"),
        TetrahedronId::from_string("g01"),
        TetrahedronId::from_string("g02"),
        TetrahedronId::from_string("g03"),
        TetrahedronId::from_string("g10"),
        TetrahedronId::from_string("g11"),
        TetrahedronId::from_string("g12"),
        TetrahedronId::from_string("g13"),
        TetrahedronId::from_string("g20"),
        TetrahedronId::from_string("g21"),
        TetrahedronId::from_string("g22"),
        TetrahedronId::from_string("g23"),
        TetrahedronId::from_string("g30"),
        TetrahedronId::from_string("g31"),
        TetrahedronId::from_string("g32"),
        TetrahedronId::from_string("g33"),
        TetrahedronId::from_string("h00"),
        TetrahedronId::from_string("h01"),
        TetrahedronId::from_string("h02"),
        TetrahedronId::from_string("h03"),
        TetrahedronId::from_string("h10"),
        TetrahedronId::from_string("h11"),
        TetrahedronId::from_string("h12"),
        TetrahedronId::from_string("h13"),
        TetrahedronId::from_string("h20"),
        TetrahedronId::from_string("h21"),
        TetrahedronId::from_string("h22"),
        TetrahedronId::from_string("h23"),
        TetrahedronId::from_string("h30"),
        TetrahedronId::from_string("h31"),
        TetrahedronId::from_string("h32"),
        TetrahedronId::from_string("h33"),
        TetrahedronId::from_string("i00"),
        TetrahedronId::from_string("i01"),
        TetrahedronId::from_string("i02"),
        TetrahedronId::from_string("i03"),
        TetrahedronId::from_string("i10"),
        TetrahedronId::from_string("i11"),
        TetrahedronId::from_string("i12"),
        TetrahedronId::from_string("i13"),
        TetrahedronId::from_string("i20"),
        TetrahedronId::from_string("i21"),
        TetrahedronId::from_string("i22"),
        TetrahedronId::from_string("i23"),
        TetrahedronId::from_string("i30"),
        TetrahedronId::from_string("i31"),
        TetrahedronId::from_string("i32"),
        TetrahedronId::from_string("i33"),
        TetrahedronId::from_string("j00"),
        TetrahedronId::from_string("j01"),
        TetrahedronId::from_string("j02"),
        TetrahedronId::from_string("j03"),
        TetrahedronId::from_string("j10"),
        TetrahedronId::from_string("j11"),
        TetrahedronId::from_string("j12"),
        TetrahedronId::from_string("j13"),
        TetrahedronId::from_string("j20"),
        TetrahedronId::from_string("j21"),
        TetrahedronId::from_string("j22"),
        TetrahedronId::from_string("j23"),
        TetrahedronId::from_string("j30"),
        TetrahedronId::from_string("j31"),
        TetrahedronId::from_string("j32"),
        TetrahedronId::from_string("j33"),
        TetrahedronId::from_string("k00"),
        TetrahedronId::from_string("k01"),
        TetrahedronId::from_string("k02"),
        TetrahedronId::from_string("k03"),
        TetrahedronId::from_string("k10"),
        TetrahedronId::from_string("k11"),
        TetrahedronId::from_string("k12"),
        TetrahedronId::from_string("k13"),
        TetrahedronId::from_string("k20"),
        TetrahedronId::from_string("k21"),
        TetrahedronId::from_string("k22"),
        TetrahedronId::from_string("k23"),
        TetrahedronId::from_string("k30"),
        TetrahedronId::from_string("k31"),
        TetrahedronId::from_string("k32"),
        TetrahedronId::from_string("k33"),
        TetrahedronId::from_string("l00"),
        TetrahedronId::from_string("l01"),
        TetrahedronId::from_string("l02"),
        TetrahedronId::from_string("l03"),
        TetrahedronId::from_string("l10"),
        TetrahedronId::from_string("l11"),
        TetrahedronId::from_string("l12"),
        TetrahedronId::from_string("l13"),
        TetrahedronId::from_string("l20"),
        TetrahedronId::from_string("l21"),
        TetrahedronId::from_string("l22"),
        TetrahedronId::from_string("l23"),
        TetrahedronId::from_string("l30"),
        TetrahedronId::from_string("l31"),
        TetrahedronId::from_string("l32"),
        TetrahedronId::from_string("l33"),
        TetrahedronId::from_string("m00"),
        TetrahedronId::from_string("m01"),
        TetrahedronId::from_string("m02"),
        TetrahedronId::from_string("m03"),
        TetrahedronId::from_string("m10"),
        TetrahedronId::from_string("m11"),
        TetrahedronId::from_string("m12"),
        TetrahedronId::from_string("m13"),
        TetrahedronId::from_string("m20"),
        TetrahedronId::from_string("m21"),
        TetrahedronId::from_string("m22"),
        TetrahedronId::from_string("m23"),
        TetrahedronId::from_string("m30"),
        TetrahedronId::from_string("m31"),
        TetrahedronId::from_string("m32"),
        TetrahedronId::from_string("m33"),
        TetrahedronId::from_string("n00"),
        TetrahedronId::from_string("n01"),
        TetrahedronId::from_string("n02"),
        TetrahedronId::from_string("n03"),
        TetrahedronId::from_string("n10"),
        TetrahedronId::from_string("n11"),
        TetrahedronId::from_string("n12"),
        TetrahedronId::from_string("n13"),
        TetrahedronId::from_string("n20"),
        TetrahedronId::from_string("n21"),
        TetrahedronId::from_string("n22"),
        TetrahedronId::from_string("n23"),
        TetrahedronId::from_string("n30"),
        TetrahedronId::from_string("n31"),
        TetrahedronId::from_string("n32"),
        TetrahedronId::from_string("n33"),
        TetrahedronId::from_string("o00"),
        TetrahedronId::from_string("o01"),
        TetrahedronId::from_string("o02"),
        TetrahedronId::from_string("o03"),
        TetrahedronId::from_string("o10"),
        TetrahedronId::from_string("o11"),
        TetrahedronId::from_string("o12"),
        TetrahedronId::from_string("o13"),
        TetrahedronId::from_string("o20"),
        TetrahedronId::from_string("o21"),
        TetrahedronId::from_string("o22"),
        TetrahedronId::from_string("o23"),
        TetrahedronId::from_string("o30"),
        TetrahedronId::from_string("o31"),
        TetrahedronId::from_string("o32"),
        TetrahedronId::from_string("o33"),
        TetrahedronId::from_string("p00"),
        TetrahedronId::from_string("p01"),
        TetrahedronId::from_string("p02"),
        TetrahedronId::from_string("p03"),
        TetrahedronId::from_string("p10"),
        TetrahedronId::from_string("p11"),
        TetrahedronId::from_string("p12"),
        TetrahedronId::from_string("p13"),
        TetrahedronId::from_string("p20"),
        TetrahedronId::from_string("p21"),
        TetrahedronId::from_string("p22"),
        TetrahedronId::from_string("p23"),
        TetrahedronId::from_string("p30"),
        TetrahedronId::from_string("p31"),
        TetrahedronId::from_string("p32"),
        TetrahedronId::from_string("p33"),
        TetrahedronId::from_string("q00"),
        TetrahedronId::from_string("q01"),
        TetrahedronId::from_string("q02"),
        TetrahedronId::from_string("q03"),
        TetrahedronId::from_string("q10"),
        TetrahedronId::from_string("q11"),
        TetrahedronId::from_string("q12"),
        TetrahedronId::from_string("q13"),
        TetrahedronId::from_string("q20"),
        TetrahedronId::from_string("q21"),
        TetrahedronId::from_string("q22"),
        TetrahedronId::from_string("q23"),
        TetrahedronId::from_string("q30"),
        TetrahedronId::from_string("q31"),
        TetrahedronId::from_string("q32"),
        TetrahedronId::from_string("q33"),
        TetrahedronId::from_string("r00"),
        TetrahedronId::from_string("r01"),
        TetrahedronId::from_string("r02"),
        TetrahedronId::from_string("r03"),
        TetrahedronId::from_string("r10"),
        TetrahedronId::from_string("r11"),
        TetrahedronId::from_string("r12"),
        TetrahedronId::from_string("r13"),
        TetrahedronId::from_string("r20"),
        TetrahedronId::from_string("r21"),
        TetrahedronId::from_string("r22"),
        TetrahedronId::from_string("r23"),
        TetrahedronId::from_string("r30"),
        TetrahedronId::from_string("r31"),
        TetrahedronId::from_string("r32"),
        TetrahedronId::from_string("r33"),
        TetrahedronId::from_string("s00"),
        TetrahedronId::from_string("s01"),
        TetrahedronId::from_string("s02"),
        TetrahedronId::from_string("s03"),
        TetrahedronId::from_string("s10"),
        TetrahedronId::from_string("s11"),
        TetrahedronId::from_string("s12"),
        TetrahedronId::from_string("s13"),
        TetrahedronId::from_string("s20"),
        TetrahedronId::from_string("s21"),
        TetrahedronId::from_string("s22"),
        TetrahedronId::from_string("s23"),
        TetrahedronId::from_string("s30"),
        TetrahedronId::from_string("s31"),
        TetrahedronId::from_string("s32"),
        TetrahedronId::from_string("s33"),
        TetrahedronId::from_string("t00"),
        TetrahedronId::from_string("t01"),
        TetrahedronId::from_string("t02"),
        TetrahedronId::from_string("t03"),
        TetrahedronId::from_string("t10"),
        TetrahedronId::from_string("t11"),
        TetrahedronId::from_string("t12"),
        TetrahedronId::from_string("t13"),
        TetrahedronId::from_string("t20"),
        TetrahedronId::from_string("t21"),
        TetrahedronId::from_string("t22"),
        TetrahedronId::from_string("t23"),
        TetrahedronId::from_string("t30"),
        TetrahedronId::from_string("t31"),
        TetrahedronId::from_string("t32"),
        TetrahedronId::from_string("t33"),
    ];

    region_ids
}