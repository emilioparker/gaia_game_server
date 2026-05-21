use super::Definition;

#[derive(Debug, Clone, Default)]
pub struct StatInitMax
{
    pub init: u16,
    pub max: u16,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct Profession
{
    pub id: u8,
    pub profession: String,
    pub strength_stat_init_max: String,
    pub vitality_stat_init_max: String,
    pub agility_stat_init_max: String,
    pub will_stat_init_max: String,
    #[serde(skip)]
    pub strength: StatInitMax,
    #[serde(skip)]
    pub vitality: StatInitMax,
    #[serde(skip)]
    pub agility: StatInitMax,
    #[serde(skip)]
    pub will: StatInitMax,
}

impl Profession
{
    fn parse_init_max(value: &str) -> StatInitMax
    {
        let parts: Vec<&str> = value.split('/').collect();
        let init: u16 = parts[0].parse().unwrap();
        let max: u16 = parts[1].parse().unwrap();
        StatInitMax { init, max }
    }
}

impl Definition for Profession
{
    fn fill_details(&mut self)
    {
        self.strength = Self::parse_init_max(&self.strength_stat_init_max);
        self.vitality = Self::parse_init_max(&self.vitality_stat_init_max);
        self.agility = Self::parse_init_max(&self.agility_stat_init_max);
        self.will = Self::parse_init_max(&self.will_stat_init_max);
    }
}
