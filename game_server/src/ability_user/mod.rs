use crate::definitions::definitions_container::Definitions;


pub mod attack;
pub mod attack_result;

pub trait AbilityUser
{
    fn get_health(&self) -> i32;
    fn get_constitution(&self, definition: &Definitions) -> i32;

    fn update_health(&mut self, new_health : i32, definition: &Definitions);
    fn get_total_attack(&self, card_id : u32, definition: &Definitions) -> u16;
    fn get_total_defense(&self, definition: &Definitions) -> u16;

    fn calculate_stat(base : u16, points : u8, class_multiplier:f32, efficiency:f32) -> u16
    {
        (base as f32 + (points as f32) * class_multiplier * efficiency).round() as u16
    }
}