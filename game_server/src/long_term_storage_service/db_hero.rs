use bson::oid::ObjectId;
use serde::Serialize;
use serde::Deserialize;

use crate::buffs::buff::Buff;
use crate::hero::hero_card_inventory::CardItem;
use crate::hero::hero_equipment_inventory::EquipmentItem;
use crate::hero::hero_inventory::InventoryItem;
use crate::hero::hero_skill_inventory::SkillData;


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
    pub profession: u8,
    pub character_name: String,
    pub position:String,
    pub vertex_id:i32,

    pub action: u8,
    pub flags:u8,
    pub inventory: Vec<StoredInventoryItem>,
    pub card_inventory: Vec<StoredCardItem>,
    pub equipment_inventory: Vec<StoredEquipmentItem>,

    pub level:u8,
    pub experience:u32,
    pub available_skill_points:u8, // used for stats
    pub weapon:u8,

    // stats
    pub strength_stat: u16,
    pub endurance_stat: u16,
    pub agility_stat: u16,
    pub will_stat: u16,

    // stats
    pub health: u16,
    pub mana: u16,
    pub stamina: u16,
    pub buffs: Vec<StoredBuff>,
    pub skills: Vec<StoredSkillItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredInventoryItem{
    pub item_id : u32,
    pub equipped : u8,
    pub amount : u16
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredCardItem
{
    pub item_definition_id : u16,
    pub item_unique_id : u32,
    pub equipped : u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredEquipmentItem
{
    pub item_definition_id : u16,
    pub item_unique_id : u32,
    pub equipped : u8,
}


impl From<InventoryItem> for StoredInventoryItem
{
    fn from(item: InventoryItem) -> Self
    {
        StoredInventoryItem { item_id: item.item_id, equipped: item.equipped, amount: item.amount }
    }
}

impl From<CardItem> for StoredCardItem
{
    fn from(item: CardItem) -> Self
    {
        StoredCardItem { item_definition_id: item.card_definition_id, equipped: item.slot, item_unique_id: item.card_unique_id}
    }
}

impl From<EquipmentItem> for StoredEquipmentItem
{
    fn from(item: EquipmentItem) -> Self
    {
        StoredEquipmentItem { item_definition_id: item.equipment_definition_id, equipped: item.slot, item_unique_id: item.equipment_unique_id }
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
