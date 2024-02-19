use std::collections::HashMap;

use super::{character_progression::CharacterProgression, definition_versions::DefinitionVersion, main_paths::MapPath, mob_progression::MobProgression, props_data::PropData};


#[derive(Debug, Clone)]
pub struct Definitions
{
    pub character_progression : Vec<CharacterProgression>,
    pub mob_progression : Vec<MobProgression>,
    pub props : Vec<PropData>,
    pub main_paths : Vec<MapPath>,
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
}