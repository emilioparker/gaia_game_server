use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Item 
{
    pub item_id: u32,
    pub cost: u16,
    pub usage:u8, // 0 means heal, 1 means xp
    pub equip_slot:u8, // 0 means not equippable, 1 is for the deck, the rest is for equipment.
    pub benefit:u16,
    pub store_location:String,
    pub item_name:String,
    pub item_description:String,
    pub image:String,
}

pub enum ItemUsage
{
    Heal = 1,
    AddXp = 2,
}


impl Definition for Item
{
    fn fill_details(&mut self)
    {
    }
}

// id,name,type,area,item,attack