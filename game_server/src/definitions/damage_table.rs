use std::collections::HashMap;
use super::Definition;

// // outer key: damage_type  inner key: (min_attack, max_attack)
// pub type DamageMap = HashMap<String, HashMap<(i16, i16), DamageResult>>;

#[derive(Debug, Clone)]
pub struct DamageResult
{
    pub hp: u16,
    pub critical: char, // '-' means no critical, 'A'-'E' for severity
}

impl DamageResult
{
    fn parse(s: &str) -> Self
    {
        let mut parts = s.split('|');
        let hp = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        let critical = parts.next().and_then(|v| v.chars().next()).unwrap_or('-');
        DamageResult { hp, critical }
    }

    pub fn has_critical(&self) -> bool
    {
        self.critical != '-'
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DamageTableEntry
{
    pub damage_type: String,
    pub min_attack: i16,
    pub max_attack: i16,

    pub at1:  String,
    pub at2:  String,
    pub at3:  String,
    pub at4:  String,
    pub at5:  String,
    pub at6:  String,
    pub at7:  String,
    pub at8:  String,
    pub at9:  String,
    pub at10: String,
    pub at11: String,
    pub at12: String,
    pub at13: String,
    pub at14: String,
    pub at15: String,
    pub at16: String,
    pub at17: String,
    pub at18: String,
    pub at19: String,
    pub at20: String,

    // results[i] corresponds to AT(i+1), e.g. results[0] = AT1
    #[serde(skip)]
    pub results: Vec<DamageResult>,
}

impl DamageTableEntry
{
    pub fn get_result(&self, armor_type_index: usize) -> Option<&DamageResult>
    {
        self.results.get(armor_type_index.saturating_sub(1))
    }

    // pub fn build_damage_map(damage_table: &[DamageTableEntry], armor_type_id: &str) -> DamageMap
    // {
    //     let index: usize = armor_type_id.trim_start_matches("at").parse().unwrap_or(0);
    //     let mut map: DamageMap = HashMap::new();

    //     for entry in damage_table
    //     {
    //         if let Some(result) = entry.get_result(index)
    //         {
    //             map.entry(entry.damage_type.clone())
    //                 .or_insert_with(HashMap::new)
    //                 .insert((entry.min_attack, entry.max_attack), result.clone());
    //         }
    //     }

    //     map
    // }
}

impl Definition for DamageTableEntry
{
    fn fill_details(&mut self)
    {
        let raw = [
            &self.at1,  &self.at2,  &self.at3,  &self.at4,  &self.at5,
            &self.at6,  &self.at7,  &self.at8,  &self.at9,  &self.at10,
            &self.at11, &self.at12, &self.at13, &self.at14, &self.at15,
            &self.at16, &self.at17, &self.at18, &self.at19, &self.at20,
        ];
        self.results = raw.iter().map(|s| DamageResult::parse(s)).collect();
    }
}


pub fn get_damage(damage_table: &[DamageTableEntry], damage: i16, damage_type: &str, armor_type_id: &str) -> Option<(u16, char)>
{
    let index: usize = armor_type_id.trim_start_matches("at").parse().ok()?;

    let entry = damage_table.iter().find(|e| 
    {
        e.damage_type == damage_type && damage >= e.min_attack && damage <= e.max_attack
    })?;

    let result = entry.get_result(index)?;
    Some((result.hp, result.critical))
}
