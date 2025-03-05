use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TowerDifficulty 
{
    pub tower_id: String,
    pub difficulty: i32,
    pub is_auxiliar:bool,
}

impl Definition for TowerDifficulty
{
    fn fill_details(&mut self)
    {
    }
}