use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MobProgression 
{
    pub level: u16,
    pub constitution: u16,
    pub distance_to_capital:u16,
    pub skill_points:u16,
    pub cards_data:String,
    pub cards: Option<Vec<i32>>
}

impl Definition for MobProgression
{
    fn fill_details(&mut self)
    {
        let cards_data : Vec<&str> = self.cards_data.split(';').collect();
        let card_ids :Vec<i32> = cards_data.iter().filter_map(|s| s.parse::<i32>().ok()).collect();
        self.cards = Some(card_ids);
    }
}