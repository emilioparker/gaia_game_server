use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Equipment
{
    pub id: u32,
    pub name: String,
    pub skill: String,
    pub card_name:String, // the card gives the properties when using the basic attack

    pub cooldown_factor: f32,
    pub mana_cost_factor: f32,
    pub stamina_cost_factor: f32,
    pub hit_range_factor: f32,

    pub store_location: String,
    pub store_cost : u16
}

impl Definition for Equipment
{
    fn fill_details(&mut self)
    {
    }
}
