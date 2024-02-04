use std::collections::{HashMap, HashSet};

use super::{character_progression::CharacterProgression, definition_versions::DefinitionVersion, props_data::PropData};


#[derive(Debug, Clone)]
pub struct Definitions
{
    pub character_progression : Vec<CharacterProgression>,
    pub props : Vec<PropData>,
}

#[derive(Debug, Clone)]
pub struct DefinitionsData
{
    pub definition_versions : HashMap<String, DefinitionVersion>,
    pub character_progression_data : Vec<u8>,
    pub definition_versions_data : Vec<u8>,
    pub props_data : Vec<u8>
}