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



// id,name,type,area,item,attack