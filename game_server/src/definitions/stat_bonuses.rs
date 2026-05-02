use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct StatBonus
{
    #[serde(rename = "Stat")]
    pub stat: u16,
    #[serde(rename = "Bonus")]
    pub bonus: i16,
}

impl Definition for StatBonus {}
