use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Skill
{
    pub id: u32,
    pub name: String,
}

impl Definition for Skill
{
    fn fill_details(&mut self)
    {
    }
}
