#[derive(Debug, Clone, serde::Deserialize)]
pub struct MobProgression 
{
    pub level: u16,
    pub world_y:u16,
    pub skill_points:u16,
}