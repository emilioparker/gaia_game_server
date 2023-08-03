use std::{collections::{HashMap}, sync::{Arc, atomic::{AtomicU64, AtomicU32, AtomicU16}}};

use bson::oid::ObjectId;
use tokio::sync::Mutex;

use crate::character::character_entity::CharacterEntity;

use self::{map_entity::MapEntity, tetrahedron_id::TetrahedronId};

pub mod map_entity;
pub mod tetrahedron_id;
pub mod tile_attack;


pub struct GameMap { 
    pub world_id : Option<ObjectId>,
    pub world_name : String,
    pub id_generator : AtomicU16,
    pub time : AtomicU64,
    pub regions : HashMap<TetrahedronId, Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>>,
    pub active_players: Arc<HashMap<u16, AtomicU64>>,
    pub logged_in_players: Arc<HashMap<u16, AtomicU64>>,
    pub players : Arc<Mutex<HashMap<u16, CharacterEntity>>>,
    // pub sent_packages : Arc<Mutex<HashMap<u16, CharacterEntity>>>,
}

impl GameMap {
    pub fn new(
        world_id: Option<ObjectId>,
        world_name:String,
        regions: Vec<(TetrahedronId, HashMap<TetrahedronId, MapEntity>)>,
        players : HashMap<u16, CharacterEntity>,
    ) -> GameMap
    {
        let mut arc_regions = HashMap::<TetrahedronId, Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>>::new();
        let mut region_keys = Vec::<TetrahedronId>::new();

        for (key, value) in regions.into_iter()
        {
            arc_regions.insert(key.clone(), Arc::new(Mutex::new(value)));
            region_keys.push(key);
        }

        let result = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();

        let mut active_players_set = HashMap::<u16, AtomicU64>::new();
        let mut logged_in_players_set = HashMap::<u16, AtomicU64>::new();
        let mut last_id = 0u16;

        let mut i:u16 = 0;

        while i < u16::MAX {
            i = i + 1;
            active_players_set.insert(i, AtomicU64::new(0));
            logged_in_players_set.insert(i, AtomicU64::new(0));
        }

        for player in &players
        {
            if last_id < *player.0{
                last_id = *player.0;
            }
        }

        let current_time_raw = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH);
        let current_time = current_time_raw.ok().map(|d| d.as_millis() as u64).unwrap();
        println!(" current_time {:?}", current_time);

        GameMap{
            world_id,
            world_name,
            id_generator : AtomicU16::new(last_id + 1),
            active_players: Arc::new(active_players_set),
            logged_in_players : Arc::new(logged_in_players_set),
            regions : arc_regions,
            players : Arc::new(Mutex::new(players)),
            time : AtomicU64::new(current_time),
        }
    }

    fn get_parent(&self, tetrahedron_id : &TetrahedronId) -> TetrahedronId
    {
        tetrahedron_id.get_parent(7)
    }

    pub fn get_region_from_child(&self, tetrahedron_id : &TetrahedronId) -> Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>{
        let key = self.get_parent(tetrahedron_id);
        let region = self.regions.get(&key).unwrap();
        region.clone()
    }

    pub fn get_region(&self, tetrahedron_id : &TetrahedronId) -> Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>{
        let region = self.regions.get(tetrahedron_id).unwrap();
        region.clone()
    }
}

pub fn get_region_ids(lod : u8) -> Vec<TetrahedronId>
{
    let encoded_areas : [char; 20] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't'];

    let initial_tiles : Vec<TetrahedronId> = encoded_areas.map(|l| {
        let first = l.to_string();
        TetrahedronId::from_string(&first)
    }).into_iter().collect();

    let mut regions = Vec::<TetrahedronId>::new();
    for initial in initial_tiles
    {
        get_regions(initial, lod, &mut regions);
    }
    regions
}

pub fn get_regions(initial : TetrahedronId, target_lod : u8, regions : &mut Vec<TetrahedronId>)
{
    if initial.lod == target_lod
    {
        regions.push(initial);
    }
    else {
        for index in 0..4
        {
            get_regions(initial.subdivide(index), target_lod, regions);
        }
    }
}
