#[derive(Debug, Clone, serde::Deserialize)]
pub struct MobProgression 
{
    pub level: u16,
    pub distance_to_capital:u16,
    pub skill_points:u16,
}