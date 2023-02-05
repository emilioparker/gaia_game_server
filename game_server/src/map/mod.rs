use std::{collections::{HashMap}, sync::Arc};

use tokio::sync::Mutex;

use crate::player::player_entity::PlayerEntity;

use self::{map_entity::MapEntity, tetrahedron_id::TetrahedronId};

pub mod map_entity;
pub mod tetrahedron_id;


pub struct GameMap { 
    pub region_keys : Arc<Vec<TetrahedronId>>,
    pub regions : HashMap<TetrahedronId, Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>>,
    // pub players : Arc<Mutex<HashMap<u64, PlayerEntity>>>,
}

impl GameMap {
    pub fn new(regions: Vec<(TetrahedronId, HashMap<TetrahedronId, MapEntity>)>) -> GameMap
    {
        let mut arc_regions = HashMap::<TetrahedronId, Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>>::new();
        let mut region_keys = Vec::<TetrahedronId>::new();

        for (key, value) in regions.into_iter()
        {
            arc_regions.insert(key.clone(), Arc::new(Mutex::new(value)));
            region_keys.push(key);
        }

        GameMap{
            region_keys : Arc::new(region_keys),
            regions : arc_regions,
        }
    }

    fn get_parent(&self, tetrahedron_id : &TetrahedronId) -> TetrahedronId
    {
        tetrahedron_id.get_parent(7)
        // for parent in self.region_keys.iter()
        // {
        //     let is_parent = parent.is_parent(tetrahedron_id);
        //     if is_parent {
        //         return Some(parent.clone());
        //     }
        // }

        // return None;
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
