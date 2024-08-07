use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

use crate::{buffs::buff::Buff, character::character_entity::InventoryItem};


#[derive(Serialize, Deserialize, Debug)]
pub struct StoredCharacter 
{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub world_id: Option<ObjectId>,
    pub world_name: String,
    pub player_id: Option<ObjectId>,
    pub version: u16, // 2 bytes
    pub character_id: u16,
    pub faction: u8,
    pub character_name: String,
    pub position:String,
    pub vertex_id:i32,

    pub action: u8,
    pub flags:u8,
    pub inventory: Vec<StoredInventoryItem>,

    pub level:u8,
    pub experience:u32,
    pub available_skill_points:u8, // used for stats

    // attributes
    pub strength_points: u8,
    pub defense_points: u8,
    pub intelligence_points: u8,
    pub mana_points: u8,

    // attributes
    pub strength: u16,
    pub defense: u16,
    pub intelligence: u16,
    pub mana: u16,

    // stats
    pub health: i32,
    pub buffs: Vec<StoredBuff>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredInventoryItem{
    pub item_id : u32,
    pub equipped : u8,
    pub amount : u16
}


impl From<InventoryItem> for StoredInventoryItem
{
    fn from(item: InventoryItem) -> Self
    {
        StoredInventoryItem { item_id: item.item_id, equipped: item.equipped, amount: item.amount }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredBuff
{
    pub card_id:u32,
    pub stat : u8, //1
    pub buff_amount : f32, // 4
    pub hits: u8,// 1
    pub expiration_time:u32 //4
}

impl From<Buff> for StoredBuff
{
    fn from(buff: Buff) -> Self
    {
        StoredBuff { card_id: buff.card_id, stat: buff.stat.to_byte() , buff_amount: buff.buff_amount, hits: buff.hits, expiration_time: buff.expiration_time}
    }
}
