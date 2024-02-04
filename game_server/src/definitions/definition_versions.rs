#[derive(Debug, Clone, serde::Deserialize)]
pub struct DefinitionVersion 
{
    pub key: String,
    pub version: u16
}