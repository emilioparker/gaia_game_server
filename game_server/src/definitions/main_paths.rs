use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MapPath 
{
    pub origin: String,
    pub destination: String
}
impl Definition for MapPath
{
    fn fill_details(&mut self)
    {
    }
}