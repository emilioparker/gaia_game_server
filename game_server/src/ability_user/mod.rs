use crate::definitions::definitions_container::Definitions;


pub mod attack;
pub mod attack_result;

pub trait AbilityUser
{
    fn get_mana(&self) -> u16;
    fn get_max_mana(&self, definition: &Definitions) -> u16;

    fn get_intelligence(&self) -> u16;
    fn get_max_intelligence(&self, definition: &Definitions) -> u16;

    fn get_stamina(&self) -> u16;
    fn get_max_stamina(&self, definition: &Definitions) -> u16;

    fn get_hp(&self) -> u16;
    fn get_max_hp(&self, definition: &Definitions) -> u16;

    fn get_health(&self) -> u16;

    fn update_health(&mut self, new_health : u16, definition: &Definitions);

    fn get_stats(&self) -> (u16, u16, u16, u16);

    fn get_total_attack(&self, card_id : u32, definition: &Definitions) -> i16
    {
        let (card_strenght_factor, card_vitality_factor, card_agility_factor, card_will_factor) = definition.cards
            .get(card_id as usize)
            .map_or((0f32, 0f32, 0f32, 0f32), |d| (d.strength_stat, d.vitality_stat, d.agility_stat, d.will_stat));

        let (hero_strength_stat, hero_vitality_stat, hero_agility_stat, hero_will_stat) = self.get_stats();
        let mut total_bonus = 0i16;

        if card_strenght_factor > 0.01f32
        {
            let bonus = definition.stat_bonus_table.get(hero_strength_stat as usize).map_or(0, |b| b.bonus);
            total_bonus += (bonus as f32 * card_strenght_factor).round() as i16;
        }
        if card_vitality_factor > 0.01f32
        {
            let bonus = definition.stat_bonus_table.get(hero_vitality_stat as usize).map_or(0, |b| b.bonus);
            total_bonus += (bonus as f32 * card_vitality_factor).round() as i16;
        }
        if card_agility_factor > 0.01f32
        {
            let bonus = definition.stat_bonus_table.get(hero_agility_stat as usize).map_or(0, |b| b.bonus);
            total_bonus += (bonus as f32 * card_agility_factor).round() as i16;
        }
        if card_will_factor > 0.01f32
        {
            let bonus = definition.stat_bonus_table.get(hero_will_stat as usize).map_or(0, |b| b.bonus);
            total_bonus += (bonus as f32 * card_will_factor).round() as i16;
        }

        total_bonus
    }

    fn get_total_defense(&self, definition: &Definitions) -> i16
    {
        let (_, _, agility_stat, _) = self.get_stats();
        definition.stat_bonus_table.get(agility_stat as usize).map_or(0, |b| b.bonus)
    }

    fn calculate_stat(base : u16, points : u8, class_multiplier:f32, efficiency:f32) -> u16
    {
        (base as f32 + (points as f32) * class_multiplier * efficiency).round() as u16
    }
}