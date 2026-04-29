use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Profession
{
    pub id: u8,
    pub profession: String,
}

impl Definition for Profession
{
    fn fill_details(&mut self) {}
}
