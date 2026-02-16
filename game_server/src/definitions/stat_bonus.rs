use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct StatToBonus
{
    #[serde(rename = "Stat")]
    pub stat: String,
    #[serde(rename = "Bonus")]
    pub bonus: String,
    #[serde(skip)]
    pub stat_min: u16,
    #[serde(skip)]
    pub stat_max: u16,
    #[serde(skip)]
    pub bonus_value: i16,
}

impl Definition for StatToBonus
{
    fn fill_details(&mut self)
    {
        let parts: Vec<&str> = self.stat.split('-').collect();
        self.stat_min = parts[0].parse().unwrap();
        self.stat_max = parts[1].parse().unwrap();
        self.bonus_value = self.bonus.parse().unwrap();
    }
}

pub fn get_stat_bonus(stat_value: u16, stat_bonuses: &Vec<StatToBonus>) -> i16
{
    for entry in stat_bonuses
    {
        if stat_value >= entry.stat_min && stat_value <= entry.stat_max
        {
            return entry.bonus_value;
        }
    }
    0
}
