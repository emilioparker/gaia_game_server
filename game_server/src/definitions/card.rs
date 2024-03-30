#[derive(Debug, Clone, serde::Deserialize)]
pub struct Card 
{
    pub id: u32,
    pub name: String,
    pub asset: String,
    pub rank:u8,
    pub strength_factor:f32,
    pub defense_factor:f32,
    pub store_location: String,
    pub cost: u16
}