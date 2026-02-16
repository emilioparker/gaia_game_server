use std::collections::HashMap;
use super::Definition;

#[derive(Debug, Clone)]
pub struct SkillCostAndPoints
{
    pub points: u16,
    pub first_rank_cost: u16,
    pub second_rank_cost: u16,
}

impl Default for SkillCostAndPoints
{
    fn default() -> Self
    {
        SkillCostAndPoints { points: 0, first_rank_cost: 0, second_rank_cost: 0 }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Profession
{
    pub id: u8,
    pub profession: String,
    pub pugilist_skill: String,
    pub sword_skill: String,
    pub bow_skill: String,
    pub staff_skill: String,
    pub body_development_skill: String,
    pub core_development_skill: String,
    #[serde(skip)]
    pub skill_costs: HashMap<String, SkillCostAndPoints>,
}

impl Profession
{
    // parses a value in the form "points|first_rank_cost/second_rank_cost"
    fn parse_skill_value(value: &str) -> SkillCostAndPoints
    {
        let parts: Vec<&str> = value.split('|').collect();
        let points: u16 = parts[0].parse().unwrap();
        let costs: Vec<&str> = parts[1].split('/').collect();
        let first_rank_cost: u16 = costs[0].parse().unwrap();
        let second_rank_cost: u16 = costs[1].parse().unwrap();
        SkillCostAndPoints { points, first_rank_cost, second_rank_cost }
    }

    pub fn get_skill_cost(&self, skill_name: &str) -> Option<&SkillCostAndPoints>
    {
        self.skill_costs.get(skill_name)
    }
}

impl Definition for Profession
{
    fn fill_details(&mut self)
    {
        self.skill_costs = HashMap::from([
            ("pugilist_skill".to_string(), Self::parse_skill_value(&self.pugilist_skill)),
            ("sword_skill".to_string(), Self::parse_skill_value(&self.sword_skill)),
            ("bow_skill".to_string(), Self::parse_skill_value(&self.bow_skill)),
            ("staff_skill".to_string(), Self::parse_skill_value(&self.staff_skill)),
            ("body_development_skill".to_string(), Self::parse_skill_value(&self.body_development_skill)),
            ("core_development_skill".to_string(), Self::parse_skill_value(&self.core_development_skill)),
        ]);
    }
}
