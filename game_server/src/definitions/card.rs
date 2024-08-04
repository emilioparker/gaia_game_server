use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Card 
{
    pub id: u32,
    pub name: String,
    pub card_type: String,
    pub icon: String,
    pub asset: String,
    pub rank:u8,
    pub strength_factor:f32,
    pub defense_factor:f32,
    pub equip_slot:u8, // 0 means not equippable, 1 is for the deck, the rest is for equipment.
    pub store_location: String,
    pub cost: u16,
    pub duration_time:f32,
    pub hits:u8,
    pub cooldown:f32,
    pub cast_range:f32,
    pub hit_range:f32,
}

impl Definition for Card
{
    fn fill_details(&mut self)
    {
    }
}