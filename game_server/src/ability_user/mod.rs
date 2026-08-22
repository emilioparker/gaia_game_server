use crate::definitions::card::CardSkill;
use crate::definitions::definitions_container::Definitions;
use crate::hero::hero_skill_inventory::SkillData;


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

    fn get_stats(&self, definition: &Definitions) -> (u16, u16, u16, u16);

    fn get_armor_type(&self, definition: &Definitions) -> String;

    fn get_profession(&self) -> u8;

    fn get_skills(&self) -> &[SkillData];

    fn get_total_attack(&self, card_id : u32, definition: &Definitions) -> i16
    {
        let mut discarded_summary = String::new();
        self.get_total_attack_with_summary(card_id, definition, &mut discarded_summary)
    }

    fn get_total_attack_with_summary(&self, card_id : u32, definition: &Definitions, summary: &mut String) -> i16
    {
        let (card_strenght_factor, card_vitality_factor, card_agility_factor, card_will_factor) = definition.cards
            .get(card_id as usize)
            .map_or((0f32, 0f32, 0f32, 0f32), |d| (d.strength_stat, d.vitality_stat, d.agility_stat, d.will_stat));

        summary.push_str(&format!(
            "   Card details from:  {card_id} factors -> strength:{card_strenght_factor:.2} vitality:{card_vitality_factor:.2} agility:{card_agility_factor:.2} will:{card_will_factor:.2}\n"
        ));

        let (hero_strength_stat, hero_vitality_stat, hero_agility_stat, hero_will_stat) = self.get_stats(definition);
        let mut total_bonus = 0i16;

        if card_strenght_factor > 0.01f32
        {
            let bonus = definition.stat_bonus_table.get(hero_strength_stat as usize).map_or(0, |b| b.bonus);
            let contribution = (bonus as f32 * card_strenght_factor).round() as i16;
            total_bonus += contribution;
            summary.push_str(&format!("   strength stat:{hero_strength_stat} -> stat bonus:{bonus} * card factor:{card_strenght_factor:.2} = {contribution}\n"));
        }
        if card_vitality_factor > 0.01f32
        {
            let bonus = definition.stat_bonus_table.get(hero_vitality_stat as usize).map_or(0, |b| b.bonus);
            let contribution = (bonus as f32 * card_vitality_factor).round() as i16;
            total_bonus += contribution;
            summary.push_str(&format!("   vitality stat:{hero_vitality_stat} -> stat bonus:{bonus} * card factor:{card_vitality_factor:.2} = {contribution}\n"));
        }
        if card_agility_factor > 0.01f32
        {
            let bonus = definition.stat_bonus_table.get(hero_agility_stat as usize).map_or(0, |b| b.bonus);
            let contribution = (bonus as f32 * card_agility_factor).round() as i16;
            total_bonus += contribution;
            summary.push_str(&format!("   agility stat:{hero_agility_stat} -> stat bonus:{bonus} * card factor:{card_agility_factor:.2} = {contribution}\n"));
        }
        if card_will_factor > 0.01f32
        {
            let bonus = definition.stat_bonus_table.get(hero_will_stat as usize).map_or(0, |b| b.bonus);
            let contribution = (bonus as f32 * card_will_factor).round() as i16;
            total_bonus += contribution;
            summary.push_str(&format!("   will stat:{hero_will_stat} -> stat bonus:{bonus} * card factor:{card_will_factor:.2} = {contribution}\n"));
        }

        summary.push_str(&format!("-- Offensive Bonus = {total_bonus}\n"));

        total_bonus
    }

    fn get_skill_bonus(&self, card_id : u32, definition: &Definitions) -> u16
    {
        let mut discarded_summary = String::new();
        self.get_skill_bonus_with_summary(card_id, definition, &mut discarded_summary)
    }

    fn get_skill_bonus_with_summary(&self, card_id : u32, definition: &Definitions, summary: &mut String) -> u16
    {
        let card_skill_list = definition.cards
            .get(card_id as usize)
            .map_or(&[] as &[CardSkill], |d| d.skill_list.as_slice());

        let profession = self.get_profession();
        let mut total_points = 0u16;

        for skill_data in self.get_skills()
        {
            let Some(skill_definition) = definition.skills_by_id.get(skill_data.id as usize) else { continue; };

            let Some(card_skill) = card_skill_list.iter().find(|cs| cs.name == skill_definition.name) else { continue; };

            let points = skill_definition.get_skill_cost_and_points_by_id(profession).map_or(0, |c| c.points);
            let contribution = (points as f32 * card_skill.contribution * skill_data.rank as f32).round() as u16;
            total_points += contribution;

            summary.push_str(&format!(
                "   skill '{}' points:{points} * contribution:{:.2} * rank:{} = {contribution}\n",
                skill_definition.name, card_skill.contribution, skill_data.rank
            ));
        }

        summary.push_str(&format!("-- Skill Bonus = {total_points}\n"));

        total_points
    }

    fn get_total_defense(&self, definition: &Definitions) -> i16
    {
        let (_, _, agility_stat, _) = self.get_stats(definition);
        definition.stat_bonus_table.get(agility_stat as usize).map_or(0, |b| b.bonus)
    }

    fn calculate_stat(base : u16, points : u8, class_multiplier:f32, efficiency:f32) -> u16
    {
        (base as f32 + (points as f32) * class_multiplier * efficiency).round() as u16
    }
}