use std::{sync::{Arc, atomic::{AtomicU64, AtomicU16}}, collections::{HashMap, HashSet}};

use bson::oid::ObjectId;
use tokio::sync::Mutex;

use crate::{definitions::definitions_container::Definitions, hero::hero_entity::HeroEntity, kingdom::kingdom_entity::KingdomEntity, mob::mob_entity::MobEntity, tower::tower_entity::TowerEntity};

use self::{map_entity::MapEntity, tetrahedron_id::TetrahedronId};

pub mod map_entity;
pub mod tetrahedron_id;


pub struct GameMap
{ 
    pub world_id : Option<ObjectId>,
    pub world_name : String,
    pub id_generator : AtomicU16,
    pub definitions : Definitions,
    pub regions : HashMap<TetrahedronId, Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>>,
    pub mobs : HashMap<TetrahedronId, Arc<Mutex<HashMap<u32, MobEntity>>>>,
    pub mob_positions : HashMap<TetrahedronId, Arc<Mutex<HashSet<TetrahedronId>>>>,
    pub active_players: Arc<HashMap<u16, AtomicU64>>,
    pub logged_in_players: Vec<AtomicU64>,
    pub character : Arc<Mutex<HashMap<u16, HeroEntity>>>,
    pub towers : Arc<Mutex<HashMap<TetrahedronId, TowerEntity>>>,
    pub kingdomes : Arc<Mutex<HashMap<TetrahedronId, KingdomEntity>>>,
}

impl GameMap 
{
    pub fn new(
        world_id: Option<ObjectId>,
        world_name: String,
        definitions: Definitions,
        regions: Vec<(TetrahedronId, HashMap<TetrahedronId, MapEntity>)>,
        players : HashMap<u16, HeroEntity>,
        towers : HashMap<TetrahedronId, TowerEntity>,
        kingdomes : HashMap<TetrahedronId, KingdomEntity>,
    ) -> GameMap
    {
        let mut arc_regions = HashMap::<TetrahedronId, Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>>::new();
        let mut arc_mobs= HashMap::<TetrahedronId, Arc<Mutex<HashMap<u32, MobEntity>>>>::new();
        let mut arc_mob_positions= HashMap::<TetrahedronId, Arc<Mutex<HashSet<TetrahedronId>>>>::new();
        let mut region_keys = Vec::<TetrahedronId>::new();

        for (key, value) in regions.into_iter()
        {
            arc_regions.insert(key.clone(), Arc::new(Mutex::new(value)));
            arc_mobs.insert(key.clone(), Arc::new(Mutex::new(HashMap::new())));
            arc_mob_positions.insert(key.clone(), Arc::new(Mutex::new(HashSet::new())));
            region_keys.push(key);
        }

        let mut active_players_set = HashMap::<u16, AtomicU64>::new();
        let mut logged_in_players_set : Vec<AtomicU64> = Vec::with_capacity(u16::MAX as usize);
        let mut last_id = 0u16;

        let mut i:u16 = 0;

        while i < u16::MAX 
        {
            i = i + 1;
            active_players_set.insert(i, AtomicU64::new(0));
            logged_in_players_set.push(AtomicU64::new(0));
        }

        for player in &players
        {
            if last_id < *player.0
            {
                last_id = *player.0;
            }
        }

        // let current_time_raw = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH);
        // let current_time = current_time_raw.ok().map(|d| d.as_millis() as u64).unwrap();
        // cli_log::info!(" current_time {:?}", current_time);

        GameMap
        {
            world_id,
            world_name,
            id_generator : AtomicU16::new(last_id + 1),
            definitions,
            active_players: Arc::new(active_players_set),
            logged_in_players : logged_in_players_set,
            regions : arc_regions,
            mobs: arc_mobs,
            mob_positions: arc_mob_positions,
            character : Arc::new(Mutex::new(players)),
            towers : Arc::new(Mutex::new(towers)),
            kingdomes : Arc::new(Mutex::new(kingdomes)),
        }
    }

    fn get_parent(&self, tetrahedron_id : &TetrahedronId) -> TetrahedronId
    {
        tetrahedron_id.get_parent(7)
    }

    pub fn get_region_from_child(&self, tetrahedron_id : &TetrahedronId) -> Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>
    {
        let key = self.get_parent(tetrahedron_id);
        let region = self.regions.get(&key).unwrap();
        region.clone()
    }

    pub fn get_mob_region_from_child(&self, tetrahedron_id : &TetrahedronId) -> Arc<Mutex<HashMap<u32, MobEntity>>>
    {
        let key = self.get_parent(tetrahedron_id);
        let mob_region = self.mobs.get(&key).unwrap();
        mob_region.clone()
    }

    pub fn get_mob_positions_region_from_child(&self, tetrahedron_id : &TetrahedronId) -> Arc<Mutex<HashSet<TetrahedronId>>>
    {
        let key = self.get_parent(tetrahedron_id);
        let mob_region = self.mob_positions.get(&key).unwrap();
        mob_region.clone()
    }

    pub fn get_region(&self, tetrahedron_id : &TetrahedronId) -> Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>
    {
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
