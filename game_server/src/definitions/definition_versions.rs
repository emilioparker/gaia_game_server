use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DefinitionVersion 
{
    pub key: String,
    pub version: u16
}

impl Definition for DefinitionVersion
{
    fn fill_details(&mut self)
    {
    }
}