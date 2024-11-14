use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Weapon 
{
    pub id: u32,
    pub name: String,
    pub weapon_type: String,
    pub card_name:String,
    pub icon: String,
    pub store_location: String,
    pub store_cost : u16
}

impl Definition for Weapon
{
    fn fill_details(&mut self)
    {
    }
}