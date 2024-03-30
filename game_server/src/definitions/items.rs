#[derive(Debug, Clone, serde::Deserialize)]
pub struct Item 
{
    pub item_id: u32,
    pub cost: u16,
    pub usage:u8, // 0 means heal, 1 means xp
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



// id,name,type,area,item,attack