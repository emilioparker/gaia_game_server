use std::collections::HashMap;

use crate::{buffs::buff, map::tetrahedron_id::TetrahedronId};

use super::{buffs_data::BuffData, card::Card, character_progression::CharacterProgression, definition_versions::DefinitionVersion, items::Item, main_paths::MapPath, mob_progression::MobProgression, mobs_data::MobData, props_data::PropData, tower_difficulty::TowerDifficulty, weapons::Weapon};


#[derive(Debug, Clone)]
pub struct Definitions
{
    pub regions_by_code: [TetrahedronId; 321],
    pub regions_by_id: HashMap<TetrahedronId, u16>,
    pub character_progression : Vec<CharacterProgression>,
    pub mob_progression : Vec<MobProgression>,
    pub mob_progression_by_mob : Vec<Vec<MobProgression>>,
    pub props : Vec<PropData>,
    pub main_paths : Vec<MapPath>,
    pub towers_difficulty : Vec<TowerDifficulty>,
    pub items : Vec<Item>,
    pub cards : Vec<Card>,
    pub mobs : Vec<MobData>,
    pub buffs : HashMap<String, BuffData>,
    pub buffs_by_code : Vec<BuffData>,
    pub weapons : Vec<Weapon>,
}

#[derive(Debug, Clone)]
pub struct DefinitionsData
{
    pub definition_versions : HashMap<String, DefinitionVersion>,
    pub character_progression_data : Vec<u8>,
    pub mob_progression_data : Vec<u8>,
    pub definition_versions_data : Vec<u8>,
    pub props_data : Vec<u8>,
    pub main_paths_data : Vec<u8>,
    pub towers_difficulty_data : Vec<u8>,
    pub items_data : Vec<u8>,
    pub cards_data : Vec<u8>,
    pub mobs_data : Vec<u8>,
    pub buffs_data : Vec<u8>,
    pub weapons_data : Vec<u8>,
}

impl Definitions 
{
    // used by the test_client ignores the protocol byte.
    pub fn get_buff(&self, id : &String) -> Option<&BuffData>
    {
        self.buffs.get(id)
    }

    pub fn get_buff_by_code(&self, id : u8) -> Option<&BuffData>
    {
        self.buffs_by_code.get(id as usize)
    }
}