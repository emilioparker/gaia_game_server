use std::collections::HashMap;

use crate::definitions::stat_bonuses::StatBonus;
use crate::map::tetrahedron_id::TetrahedronId;

use super::buffs_data::BuffData;
use super::card::Card;
use super::character_progression::CharacterProgression;
use super::definition_versions::DefinitionVersion;
use super::equipment::Equipment;
use super::professions::Profession;
use super::items::Item;
use super::main_paths::MapPath;
use super::mob_progression::MobProgression;
use super::mobs_data::MobData;
use super::props_data::PropData;
use super::skills::Skill;
use super::stat_gains::StatGain;
use super::realms::Realm;
use super::tower_difficulty::TowerDifficulty;
use super::armor_types::ArmorType;
use super::damage_table::DamageTableEntry;


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
    pub equipment : Vec<Equipment>,
    pub skills : HashMap<String, Skill>,
    pub skills_by_id : Vec<Skill>,
    pub professions : HashMap<String, Profession>,
    pub professions_by_id : Vec<Profession>,
    pub stat_bonus_table : Vec<StatBonus>,
    pub stat_gains : Vec<StatGain>,
    pub realms : Vec<Realm>,
    pub armor_types : Vec<ArmorType>,
    pub damage_table : HashMap<String, Vec<DamageTableEntry>>,
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
    pub equipment_data : Vec<u8>,
    pub skills_data : Vec<u8>,
    pub initial_stats_data : Vec<u8>,
    pub stat_bonuses_data : Vec<u8>,
    pub stat_gains_data : Vec<u8>,
    pub realms_data : Vec<u8>,
    pub armor_types_data : Vec<u8>,
    pub damage_table_data : HashMap<String, Vec<u8>>,
}
