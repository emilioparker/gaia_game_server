use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct BuffData 
{
    pub id:String,
    pub code: u8,
    pub buff_type:String,
    pub base_value:f32,
    pub hits:u8,
    pub duration:u32,
}


impl Definition for BuffData
{
    fn fill_details(&mut self)
    {
    }
}