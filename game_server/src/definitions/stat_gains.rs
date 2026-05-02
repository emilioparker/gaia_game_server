use super::Definition;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct StatGain
{
    #[serde(rename = "Difference")]
    pub difference: u8,
    #[serde(rename = "Roll_Min")]
    pub roll_min: u8,
    #[serde(rename = "Roll_Max")]
    pub roll_max: u8,
    #[serde(rename = "Gain")]
    pub gain: u8,
}

impl Definition for StatGain {}

impl StatGain
{
    pub fn get_gain(&self, roll: u8) -> u8
    {
        if roll >= self.roll_min && roll <= self.roll_max
        {
            self.gain
        }
        else
        {
            0
        }
    }
}
