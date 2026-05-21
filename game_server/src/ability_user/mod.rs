use crate::definitions::definitions_container::Definitions;


pub mod attack;
pub mod attack_result;

pub trait AbilityUser
{
    fn get_mana(&self) -> u16;
    fn get_max_mana(&self, definition: &Definitions) -> u16;

    fn get_intelligence(&self) -> u16;
    fn get_max_intelligence(&self, definition: &Definitions) -> u16;

    fn get_endurance(&self) -> u16;
    fn get_max_endurance(&self, definition: &Definitions) -> u16;

    fn get_hp(&self) -> u16;
    fn get_max_hp(&self, definition: &Definitions) -> u16;



    fn get_health(&self) -> u16;
    fn get_hit_points(&self, definition: &Definitions) -> u16;

    fn update_health(&mut self, new_health : u16, definition: &Definitions);
    fn get_total_attack(&self, card_id : u32, definition: &Definitions) -> i16;
    fn get_total_defense(&self, definition: &Definitions) -> i16;

    fn calculate_stat(base : u16, points : u8, class_multiplier:f32, efficiency:f32) -> u16
    {
        (base as f32 + (points as f32) * class_multiplier * efficiency).round() as u16
    }
}