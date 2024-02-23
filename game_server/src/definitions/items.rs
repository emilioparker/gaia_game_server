#[derive(Debug, Clone, serde::Deserialize)]
pub struct Item 
{
    pub item_id: u16,
    pub min_cost: u16,
    pub max_cost: u16,
    pub store_location:String,
    pub item_name:String,
    pub item_description:String,
    pub image:String,
}



// id,name,type,area,item,attack