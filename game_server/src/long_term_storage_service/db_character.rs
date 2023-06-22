use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

use crate::character::character_entity::InventoryItem;


#[derive(Serialize, Deserialize, Debug)]
pub struct StoredCharacter {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub world_id: Option<ObjectId>,
    pub world_name: String,
    pub player_id: Option<ObjectId>,
    pub character_id: u16,
    pub faction: String,
    pub character_name: String,
    pub position:[f32;3],
    pub inventory: Vec<StoredInventoryItem>,
    pub constitution: u32,
    pub health: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredInventoryItem{
    pub item_id : u32,
    pub level : u8,
    pub quality : u8,
    pub amount : u16
}


impl From<InventoryItem> for StoredInventoryItem {
    fn from(item: InventoryItem) -> Self {
        StoredInventoryItem { item_id: item.item_id, level: item.level, quality: item.quality, amount: item.amount }
    }
}
