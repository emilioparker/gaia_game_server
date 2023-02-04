use bson::oid::ObjectId;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct StoredCharacter {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub world_id: Option<ObjectId>,
    pub world_name: String,
    pub region_id: String,
    pub compressed_data : bson::Bson,
}
