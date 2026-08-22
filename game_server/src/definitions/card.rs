use super::Definition;

#[derive(Debug, Clone)]
pub struct CardSkill
{
    pub name: String,
    pub contribution: f32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Card
{
    pub id: u32,
    pub name: String,
    pub card_type: String,
    pub damage_type: String,
    pub required_weapon: String,
    pub target_type: String,

    #[serde(skip)]
    pub damage_types: Vec<String>,

    pub strength_stat: f32,
    pub vitality_stat: f32,
    pub agility_stat: f32,
    pub will_stat: f32,

    pub equip_slot:u8, // 0 means not equippable, 1 is for the deck, the rest is for equipment.
    pub store_location: String,
    pub store_cost: u16,

    // el arma lo afecta
    pub cooldown:f32,
    pub mana_cost: u16,
    pub stamina_cost: u16,
    pub hit_range:f32,

    pub hits:u8,
    pub duration_time:f32,

    pub buff:String,
    pub effect_probability:f32,

    pub skills: String,

    #[serde(skip)]
    pub skill_list: Vec<CardSkill>,
}

impl Definition for Card
{
    fn fill_details(&mut self)
    {
        self.damage_types = self.damage_type
            .split(';')
            .map(|s| s.to_string())
            .collect();

        self.skill_list = self.skills
            .split(';')
            .filter_map(|entry| {
                let (name, contribution) = entry.split_once(':')?;
                Some(CardSkill {
                    name: name.to_string(),
                    contribution: contribution.parse().unwrap_or(0f32),
                })
            })
            .collect();
    }
}