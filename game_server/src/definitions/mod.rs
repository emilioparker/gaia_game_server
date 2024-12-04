pub mod character_progression;
pub mod mob_progression;
pub mod definition_versions;
pub mod definitions_container;
pub mod props_data;
pub mod main_paths;
pub mod items;
pub mod card;
pub mod mobs_data;
pub mod buffs_data;
pub mod weapons;
pub mod tower_difficulty;


pub trait Definition 
{
    fn fill_details(&mut self)
    {

    }
}