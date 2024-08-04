use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MobProgression 
{
    pub level: u16,
    pub constitution: u16,
    pub base_strength: u16,
    pub strength_points: u16,
    pub base_defense: u16,
    pub defense_points: u16,
    pub distance_to_capital:u16,
    pub cards_data:String,
    pub cards: Option<Vec<i32>>
}

// level,constitution,base_strength,strength_points,base_defense,defense_points,distance_to_capital,skill_points,cards_data

impl Definition for MobProgression
{
    fn fill_details(&mut self)
    {
        let cards_data : Vec<&str> = self.cards_data.split(';').collect();
        let card_ids :Vec<i32> = cards_data.iter().filter_map(|s| s.parse::<i32>().ok()).collect();
        self.cards = Some(card_ids);
    }
}