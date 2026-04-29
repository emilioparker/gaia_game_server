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
pub struct Skill
{
    pub id: u32,
    pub name: String,
    pub initiate: String,
    pub tank: String,
    pub warrior: String,
    pub mage: String,
    #[serde(skip)]
    pub profession_costs: HashMap<String, SkillCostAndPoints>,
}

impl Skill
{
    fn parse_skill_value(value: &str) -> SkillCostAndPoints
    {
        let parts: Vec<&str> = value.split('|').collect();
        let points: u16 = parts[0].parse().unwrap();
        let costs: Vec<&str> = parts[1].split('/').collect();
        let first_rank_cost: u16 = costs[0].parse().unwrap();
        let second_rank_cost: u16 = costs[1].parse().unwrap();
        SkillCostAndPoints { points, first_rank_cost, second_rank_cost }
    }

    pub fn get_skill_cost_and_points(&self, profession_name: &str) -> Option<&SkillCostAndPoints>
    {
        self.profession_costs.get(profession_name)
    }
}

impl Definition for Skill
{
    fn fill_details(&mut self)
    {
        self.profession_costs = HashMap::from([
            ("Initiate".to_string(), Self::parse_skill_value(&self.initiate)),
            ("Tank".to_string(), Self::parse_skill_value(&self.tank)),
            ("Warrior".to_string(), Self::parse_skill_value(&self.warrior)),
            ("Mage".to_string(), Self::parse_skill_value(&self.mage)),
        ]);
    }
}
