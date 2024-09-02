use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BuffData 
{
    pub id: u16,
    pub name:String,
    pub buff_type:String,
    pub base_value:f32,
    pub hits:u8,
    pub duration:u8,
}


impl Definition for BuffData
{
    fn fill_details(&mut self)
    {
    }
}