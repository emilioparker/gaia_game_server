use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ArmorType
{
    pub id: String,
    pub armor_type: String,
}

impl Definition for ArmorType {}
