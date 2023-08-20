use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

use crate::{tower::tower_entity::DamageByFaction, get_faction_from_code};


#[derive(Serialize, Deserialize, Debug)]
pub struct StoredTower {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub tetrahedron_id : String,
    pub world_id: Option<ObjectId>,
    pub world_name: String,
    pub version: u16, // 2 bytes
    pub faction: String,
    pub event_id : u16,
    pub damage_received_in_event: Vec<StoredDamageByFaction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredDamageByFaction
{
    pub faction: String,
    pub amount : u16,
    pub event_id : u16,
}

impl From<DamageByFaction> for StoredDamageByFaction {
    fn from(item: DamageByFaction) -> Self {
        StoredDamageByFaction { faction: get_faction_from_code(item.faction), amount: item.amount, event_id: item.event_id }
    }
}