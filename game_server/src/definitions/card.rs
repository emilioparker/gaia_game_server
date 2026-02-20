use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Card 
{
    pub id: u32,
    pub name: String,
    pub card_type: String,
    pub required_equipment: String,
    pub target_type: String,

    pub strength_stat: u16,
    pub endurance_stat: u16,
    pub agility_stat: u16,
    pub will_stat: u16,

    pub equip_slot:u8, // 0 means not equippable, 1 is for the deck, the rest is for equipment.
    pub store_location: String,
    pub store_cost: u16,

    // el arma lo afecta
    pub cooldown:f32,
    pub mana_cost: u16,
    pub stamina_cost: u16,
    pub hit_range:f32,

    pub hits:u8,
    pub duration_time:f32,

    pub buff:String,
    pub effect_probability:f32,
}

impl Definition for Card
{
    fn fill_details(&mut self)
    {
    }
}