use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PropData 
{
    pub id: u16,
    pub name:String,
    pub prop_type:String,
    pub area:u8,
    pub item:String,
    pub attack:String,
}


impl Definition for PropData
{
    fn fill_details(&mut self)
    {
    }
}

// id,name,type,area,item,attack