#[derive(Debug, Clone, serde::Deserialize)]
pub struct CharacterProgression 
{
    pub level: u16,
    pub required_xp:u32,
    pub skill_points:u16,
}