use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Realm
{
    pub realm_id: u8,
    pub realm: String,
    pub multiplier: f32,
}

impl Definition for Realm {}
