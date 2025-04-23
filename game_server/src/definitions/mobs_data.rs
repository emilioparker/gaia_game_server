use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MobData 
{
    pub id: u16,
    pub name:String,
    pub mob_type:String,
    pub item:String,
    pub area:String,
}


impl Definition for MobData
{
    fn fill_details(&mut self)
    {
    }
}