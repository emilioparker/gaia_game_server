use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

use crate::{buffs::buff::Buff, hero::{hero_card_inventory::CardItem, hero_inventory::InventoryItem, hero_skill_inventory::SkillData, hero_tower_progress::HeroTowerProgress, hero_weapon_inventory::WeaponItem}};


#[derive(Serialize, Deserialize, Debug)]
pub struct StoredHero 
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
    pub card_inventory: Vec<StoredInventoryItem>,
    pub weapon_inventory: Vec<StoredInventoryItem>,
    pub tower_progress : StoredTowerProgress,

    pub level:u8,
    pub experience:u32,
    pub available_skill_points:u8, // used for stats
    pub weapon:u8,

    // attributes
    pub development_points: u8,
    pub power_points: u8,
    pub stamina_points: u8,

    // stats
    pub strength_stat: u16,
    pub endurance_stat: u16,
    pub agility_stat: u16,
    pub will_stat: u16,

    // stats
    pub health: u16,
    pub buffs: Vec<StoredBuff>,
    pub skills: Vec<StoredSkillItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredTowerProgress
{
    pub id : String, // 6
    pub tower_floor: u32, //4
    pub start_time : u64, // 8
    pub points : u32 // 4
}

impl From<HeroTowerProgress> for StoredTowerProgress
{
    fn from(item: HeroTowerProgress) -> Self
    {
        StoredTowerProgress 
        { 
            id: item.id.to_string(), tower_floor: item.tower_floor, start_time: item.start_time, points: item.points
        }
    }
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

impl From<CardItem> for StoredInventoryItem
{
    fn from(item: CardItem) -> Self
    {
        StoredInventoryItem { item_id: item.card_id, equipped: item.equipped, amount: item.amount }
    }
}

impl From<WeaponItem> for StoredInventoryItem
{
    fn from(item: WeaponItem) -> Self
    {
        StoredInventoryItem { item_id: item.weapon_id, equipped: item.equipped, amount: item.amount }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredBuff
{
    pub buff_id : u8, //1
    pub hits: u8,// 1
    pub expiration_time:u32 //4
}

impl From<Buff> for StoredBuff
{
    fn from(buff: Buff) -> Self
    {
        StoredBuff {buff_id : buff.buff_id, hits : buff.hits, expiration_time : buff.expiration_time}
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredSkillItem
{
    pub id : u8,
    pub rank : u8,
    pub count : u8,
}

impl From<SkillData> for StoredSkillItem
{
    fn from(skill: SkillData) -> Self
    {
        StoredSkillItem { id: skill.id, rank: skill.rank, count: skill.count }
    }
}
