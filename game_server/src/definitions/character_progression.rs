use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CharacterProgression 
{
    pub level: u16,
    pub constitution: u16,
    pub required_xp:u32,
    pub skill_points:u16,
}

impl Definition for CharacterProgression
{
    fn fill_details(&mut self)
    {
    }
}