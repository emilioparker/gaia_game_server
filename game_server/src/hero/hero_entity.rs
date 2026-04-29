use std::hash::Hash;
use std::sync::atomic::AtomicU32;

use bson::oid::ObjectId;

use crate::ability_user::AbilityUser;
use crate::buffs::buff::Buff;
use crate::buffs::buff::BuffUser;
use crate::definitions::definitions_container::Definitions;
use crate::definitions::stat_bonus;
use crate::map::tetrahedron_id::TetrahedronId;

use super::hero_card_inventory::CardItem;
use super::hero_equipment_inventory::EquipmentItem;
use super::hero_inventory::InventoryItem;
use super::hero_skill_inventory::SkillData;

pub const HERO_ENTITY_SIZE: usize = 51;

pub const DASH_FLAG : u8 = 0b00000001;
pub const CHAT_FLAG : u8 = 0b00000010;

#[derive(Debug)]
pub struct HeroEntity
{
    pub object_id: Option<ObjectId>,
    pub player_id: Option<ObjectId>,
    pub version: u16, // 2 bytes
    pub hero_name: String,
    pub hero_id: u16, // 2 bytes
    pub faction:u8, // 1 byte
    pub profession:u8, // 1 byte
    pub equipped_item:u8,// 1 byte

    pub position: TetrahedronId, // 6 bytes

    // 13 bytes

    pub second_position: TetrahedronId, // not sent, when saving on the database, this on is stored. On login this on is used
    pub vertex_id:i32,// not sent, also saved in db, but only used on login to properly set the position of the player.

    pub path: [u8;6], // 6 bytes
    pub time : u32,// 4 bytes // el tiempo en que inicio el recorrido.
    pub action:u8, //1 bytes

    pub flags:u8, // 1 byte

    // 12 bytes
    
    pub inventory : Vec<InventoryItem>,// this one is not serializable  normally
    pub card_inventory : Vec<CardItem>,// this one is not serializable  normally
    pub equipment_inventory : Vec<EquipmentItem>,// this one is not serializable  normally
    pub skills : Vec<SkillData>,// this one is not serializable  normally
    pub inventory_version : u8, // 1 bytes

    pub level:u8, // 1 bytes
    pub experience:u32, // 4 bytes
    pub available_skill_points:u8, // 1 bytes used for stats

    // 7 bytes

    pub strength_stat: u16,
    pub endurance_stat: u16,
    pub agility_stat: u16,
    pub will_stat: u16,

    // 8 bytes

    // stats
    pub health: u16, // 2 bytes
    pub mana: u16, // 2 bytes
    pub stamina: u16, // 2 bytes
    pub buffs : Vec<Buff>,// this one is not serializable  normally
    pub buffs_summary : [u8;5], // this one is serialized but not saved 5 bytes

    pub card_id_generator : u32, // not serializable
    pub equipment_id_generator : u32, // not serializable

    // 11 bytes

    // 13 + 12 + 7 + 8 + 11 = 51
}

impl Clone for HeroEntity
{
    fn clone(&self) -> Self
    {
        HeroEntity
        {
            object_id: self.object_id.clone(),
            player_id: self.player_id.clone(),
            version: self.version,
            hero_name: self.hero_name.clone(),
            hero_id: self.hero_id,
            faction: self.faction,
            profession: self.profession,
            equipped_item: self.equipped_item,
            position: self.position.clone(),
            second_position: self.second_position.clone(),
            vertex_id: self.vertex_id,
            path: self.path,
            time: self.time,
            action: self.action,
            flags: self.flags,
            inventory: self.inventory.clone(),
            card_inventory: self.card_inventory.clone(),
            equipment_inventory: self.equipment_inventory.clone(),
            skills: self.skills.clone(),
            inventory_version: self.inventory_version,
            level: self.level,
            experience: self.experience,
            available_skill_points: self.available_skill_points,
            strength_stat: self.strength_stat,
            endurance_stat: self.endurance_stat,
            agility_stat: self.agility_stat,
            will_stat: self.will_stat,
            health: self.health,
            mana: self.mana,
            stamina: self.stamina,
            buffs: self.buffs.clone(),
            buffs_summary: self.buffs_summary,
            card_id_generator: 0,
            equipment_id_generator: 0,
        }
    }
}

impl HeroEntity
{
    pub fn clone_for_sending(&self) -> Self
    {
        HeroEntity
        {
            object_id: None,
            player_id: self.player_id.clone(),
            version: self.version,
            hero_name: "".to_owned(),
            hero_id: self.hero_id,
            faction: self.faction,
            profession: self.profession,
            equipped_item: self.equipped_item,
            position: self.position.clone(),
            second_position: self.second_position.clone(),
            vertex_id: self.vertex_id,
            path: self.path,
            time: self.time,
            action: self.action,
            flags: self.flags,
            inventory: Vec::new(),
            card_inventory: Vec::new(),
            equipment_inventory: Vec::new(),
            skills: Vec::new(),
            inventory_version: self.inventory_version,
            level: self.level,
            experience: self.experience,
            available_skill_points: self.available_skill_points,
            strength_stat: self.strength_stat,
            endurance_stat: self.endurance_stat,
            agility_stat: self.agility_stat,
            will_stat: self.will_stat,
            health: self.health,
            mana: self.mana,
            stamina: self.stamina,
            buffs: Vec::new(),
            buffs_summary: self.buffs_summary,
            card_id_generator: 0,
            equipment_id_generator: 0,
        }
    }
}

pub enum ItemType
{
    Material = 0,
    Card = 1,
    Equipment = 2
}

impl HeroEntity 
{
    pub fn to_bytes(&self) -> [u8;HERO_ENTITY_SIZE] 
    {
        let mut buffer = [0u8; HERO_ENTITY_SIZE];
        let mut offset = 0;
        let mut end;

        end = offset + 2;
        let player_id_bytes = u16::to_le_bytes(self.hero_id); // 2 bytes
        buffer[..end].copy_from_slice(&player_id_bytes);
        offset = end;

        end = offset + 2;
        let version_bytes = u16::to_le_bytes(self.version); // 2 bytes
        buffer[offset..end].copy_from_slice(&version_bytes);
        offset = end;

        end = offset + 1;
        buffer[offset] = self.faction;
        offset = end;

        end = offset + 1;
        buffer[offset] = self.profession;
        offset = end;
        // 6 bytes

        end = offset + 6;
        let position_tile_id_bytes = self.position.to_bytes();
        buffer[offset..end].copy_from_slice(&position_tile_id_bytes);
        offset = end;

        for path_point in self.path
        {
            end = offset + 1;
            buffer[offset] = path_point;
            offset = end;
        }

        end = offset + 4;
        let time_bytes = u32::to_le_bytes(self.time); // 4 bytes
        buffer[offset..end].copy_from_slice(&time_bytes);
        offset = end;

        // 16 bytes

        end = offset + 1;
        buffer[offset] = self.action;
        offset = end;

        end = offset + 1;
        buffer[offset] = self.flags;
        offset = end;

        end = offset + 1;
        buffer[offset] = self.inventory_version;
        offset = end;

        // 2 bytes

        end = offset + 1;
        buffer[offset] = self.level;
        offset = end;

        let xp_bytes = u32::to_le_bytes(self.experience); // 4 bytes
        end = offset + 4;
        buffer[offset..end].copy_from_slice(&xp_bytes);
        offset = end;

        end = offset + 1;
        buffer[offset] = self.available_skill_points;
        offset = end;

        end = offset + 1;
        buffer[offset] = self.equipped_item;
        offset = end;

        end = offset + 2;
        let strength_bytes = u16::to_le_bytes(self.strength_stat); // 2 bytes
        buffer[offset..end].copy_from_slice(&strength_bytes);
        offset = end;

        end = offset + 2;
        let endurance_bytes = u16::to_le_bytes(self.endurance_stat); // 2 bytes
        buffer[offset..end].copy_from_slice(&endurance_bytes);
        offset = end;

        end = offset + 2;
        let agility_bytes = u16::to_le_bytes(self.agility_stat); // 2 bytes
        buffer[offset..end].copy_from_slice(&agility_bytes);
        offset = end;

        let will_bytes = u16::to_le_bytes(self.will_stat); // 2 bytes
        end = offset + 2;
        buffer[offset..end].copy_from_slice(&will_bytes);
        offset = end;

        let health_bytes = u16::to_le_bytes(self.health); // 2 bytes
        end = offset + 2;
        buffer[offset..end].copy_from_slice(&health_bytes);
        offset = end;

        let mana_bytes = u16::to_le_bytes(self.mana); // 2 bytes
        end = offset + 2;
        buffer[offset..end].copy_from_slice(&mana_bytes);
        offset = end;

        let stamina_bytes = u16::to_le_bytes(self.stamina); // 2 bytes
        end = offset + 2;
        buffer[offset..end].copy_from_slice(&stamina_bytes);
        offset = end;

        // 5 pairs of 1 bytes, 10 bytes
        for buff_id in self.buffs_summary
        {
            end = offset + 1;
            buffer[offset] = buff_id;
            offset = end;
        }

        buffer
    }

    pub fn add_xp_from_battle(&mut self, xp:u32, definitions: &Definitions)
    {
        self.experience += xp;
        if let Some(next_level_data) = definitions.character_progression.get(self.level as usize + 1)
        {
            if next_level_data.required_xp <= self.experience
            {
                self.level += 1;
                self.available_skill_points = self.available_skill_points.wrapping_add(next_level_data.skill_points as u8);
                self.reset_skill_counts();
            }
        }
        cli_log::info!("----- add xp:{} from battle {}", xp, self.experience);
    }

    pub fn set_flag(&mut self, flag : u8, value : bool)
    {
        if value
        {
            self.flags = self.flags | flag;
        }
        else
        {
            self.flags = self.flags & !flag;
        }
    }

    pub fn get_flag_value(&mut self, flag : u8) -> bool
    {
        (self.flags & flag) != 0
    }

    pub fn get_size() -> usize 
    {
        HERO_ENTITY_SIZE
    }

}

impl Hash for HeroEntity 
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) 
    {
        self.action.hash(state);
    }
}

impl BuffUser for HeroEntity 
{
    fn get_buffs_mut(&mut self) -> &mut Vec<crate::buffs::buff::Buff> 
    {
        &mut self.buffs
    }

    fn get_buffs(&self) -> &Vec<crate::buffs::buff::Buff> 
    {
        &self.buffs
    }

    fn set_buffs(&mut self, new_buffs: Vec<crate::buffs::buff::Buff>) 
    {
        self.buffs = new_buffs;
    }

    fn get_buff_summary(&mut self) -> &mut [u8;5] 
    {
        &mut self.buffs_summary
    }
}

impl AbilityUser for HeroEntity
{
    fn get_health(&self) -> u16 
    {
        self.health
    }

    fn update_health(&mut self, new_health : u16, _definition: &Definitions) 
    {
        self.health = new_health;
    }

    fn get_constitution(&self, definition: &Definitions) -> u16 
    {
        let character_definition = definition.character_progression.get(self.level as usize).unwrap();
        character_definition.constitution
    }
    
    fn get_total_attack(&self, card_id : u32, definition: &Definitions) -> i16
    {
        let (str, end, agi, will) = definition.cards.get(card_id as usize).map_or((0u16, 0, 0, 0), |d| (d.strength_stat, d.endurance_stat, d.agility_stat, d.will_stat));

        let mut total_bonus = 0;

        // based on the card we pick the relevant stats.
        if str > 0
        {
            let bonus = stat_bonus::get_stat_bonus(self.strength_stat + str, &definition.stat_bonuses);
            total_bonus += bonus;
        }

        if end > 0
        {
            let bonus = stat_bonus::get_stat_bonus(self.endurance_stat + end, &definition.stat_bonuses);
            total_bonus += bonus;
        }

        if agi > 0
        {
            let bonus = stat_bonus::get_stat_bonus(self.agility_stat + agi, &definition.stat_bonuses);
            total_bonus += bonus;
        }

        if will > 0
        {
            let bonus = stat_bonus::get_stat_bonus(self.will_stat + will, &definition.stat_bonuses);
            total_bonus += bonus;
        }

        // based on the weapon we get the skill 
        // let equipment_definition =  definition.equipment.iter().find(|w| w.id == self.equipped_item as u32).unwrap();
        let skill_definition =  definition.skills_by_id.get(self.equipped_item as usize).unwrap();

        // based on the skill we get the current skill rank
        let rank = self.get_skill_rank(skill_definition.id as u8).unwrap_or(0);

        let profession = definition.professions_by_id.get(self.profession as usize).unwrap();

        // esto se ve mal, get skill costs and points deberia usarse cuando upgradeas un skill.
        // using the profession we calculate how much the skill gives us as points.
        let points = profession.get_skill_cost_and_points(&skill_definition.name).map_or(0, |s| s.points);
        total_bonus += (rank as u16 * points) as i16;

        // let stat = self.strength_stat;
        // let added_strength : f32 = self.buffs.iter().map(|b|
        //     {
        //         if let Some(def) = definition.get_buff_by_code(b.buff_id)
        //         {
        //             if def.buff_type == BUFF_STRENGTH
        //             {
        //                 return def.base_value;
        //             }
        //         }

        //         return 0f32;
        //     })
        //     .sum();

        // cli_log::info!(" -- calculate total attack {card_strength} stat {stat} buff {added_strength}");
        // stat + card_strength + added_strength.round() as u16

        total_bonus
    }

    fn get_total_defense(&self, _definition: &Definitions) -> i16
    {

//         Defensive Bonus (DB) =
//     Armor DB
//   + Shield Bonus
//   + Quickness Stat Bonus
//   + Adrenal / Skill Bonuses
//   + Magical Bonuses
//   + Misc Modifiers

        for _equipment in &self.equipment_inventory
        {

        }

        // let stat = self.endurance_stat;
        // let added_defense : f32 = self.buffs.iter().map(|b|
        //     {
        //         if let Some(def) = definition.get_buff_by_code(b.buff_id)
        //         {
        //             if def.buff_type == BUFF_DEFENSE
        //             {
        //                 return def.base_value;
        //             }
        //         }
        //         return 0f32;
        //     })
        //     .sum();

        // cli_log::info!("character added defense {} buffs_len: {}",added_defense, self.buffs.len());
        // stat + added_defense.round() as i16
        return 0;
    }

    // fn get_offensive_bonus(&self, definition: &Definitions) -> u16
    // {
    //     return 0;
    // }

    // fn get_defensive_bonus(&self, definition: &Definitions) -> u16
    // {
    //     return 0;
    // }
}


#[cfg(test)]
mod tests 
{
    use std::num::Wrapping;
    use std::sync::atomic::AtomicU32;


    use crate::hero::hero_entity::HERO_ENTITY_SIZE;
    use crate::hero::hero_inventory::HERO_INVENTORY_ITEM_SIZE;
    use crate::map::tetrahedron_id::TetrahedronId;

    use super::HeroEntity;


    #[test]
    fn test_enconde_ascii() {
        // いいえ
        let mut ch:char='い';
    
        cli_log::info!("ASCII value: {}",ch as u32);
        
        ch='&';
        cli_log::info!("ASCII value: {}",ch as u32);

        ch='X';
        cli_log::info!("ASCII value: {}",ch as u32); 
    }

    #[test]
    fn test_convert_string_to_array() {
        let name = "aaaa".to_string();
        let filled = format!("{: <5}", name);
        cli_log::info!("filled {}", filled);
        let name_data : Vec<u32> = filled.chars().into_iter().map(|c| c as u32).collect();

        let mut name_array = [0u32; 5];
        name_array.clone_from_slice(&name_data.as_slice()[0..5]);
        cli_log::info!("{:?}", name_array);
    }

    #[test]
    fn test_overflow()
    {
        let a = Wrapping(200u8);
        let b = Wrapping(2u8);
        let c = Wrapping(121u8);
        let d = Wrapping(15u8);
        let result = a * b * c * d;
        cli_log::info!("{result}");
        let result = c * b * d * a;
        cli_log::info!("{result}");
        let result = a * c * d * b;
        cli_log::info!("{result}");
    }

    #[test]
    fn test_add_inventory_item()
    {
        let mut entity = HeroEntity
        {
            object_id: None,
            player_id: None,
            version:1,
            hero_name: "a".to_owned(),
            hero_id: 1234,
            faction:0,
            profession:0,
            action: 0,
            flags:0,
            position: TetrahedronId::default(),
            second_position: TetrahedronId::default(),
            vertex_id:-1,
            path:[0,0,0,0,0,0],
            time:0,
            inventory: Vec::new(),
            card_inventory: Vec::new(),
            equipment_inventory: Vec::new(),
            skills: Vec::new(),
            inventory_version: 1,
            health: 0,
            mana: 0,
            level: 1,
            experience: 0,
            available_skill_points: 0,
            equipped_item:0,
            strength_stat: 0,
            endurance_stat: 0,
            agility_stat: 0,
            will_stat: 0,
            buffs: Vec::new(),
            buffs_summary: [0,0,0,0,0],
            stamina: 0,
            card_id_generator: 0,
            equipment_id_generator: 0,
        };

        entity.add_inventory_item(super::InventoryItem { item_id: 1, equipped: 0, amount: 1 });
        entity.add_inventory_item(super::InventoryItem { item_id: 1, equipped: 0, amount: 2 });

        assert!(entity.inventory.len() == 1);

        let item = entity.inventory.iter().next().unwrap();
        assert!(item.amount == 3);
        entity.add_inventory_item(super::InventoryItem { item_id: 2, equipped: 1, amount: 2 });
        assert!(entity.inventory.len() == 2);
        cli_log::info!("{:?}", entity.inventory);
    }

    #[test]
    fn test_encode_inventory_item()
    {

        let item = super::InventoryItem { item_id: 1, equipped: 1, amount: 1 };
        let buffer = item.to_bytes();

        assert!(buffer.len() == HERO_INVENTORY_ITEM_SIZE);
    }

    #[test]
    fn test_encode_character()
    {

        let char = HeroEntity{
            object_id: None,
            player_id: None,
            version: 1,
            hero_name: "Park".to_string(),
            hero_id: 2,
            faction: 0,
            profession: 0,
            position: TetrahedronId::default(),
            second_position: TetrahedronId::default(),
            vertex_id:-1,
            path:[0,0,0,0,0,0],
            time:0,
            action: 1,
            flags:0,
            inventory: Vec::new(),
            card_inventory: Vec::new(),
            equipment_inventory: Vec::new(),
            skills: Vec::new(),
            inventory_version: 10,
            level: 0,
            experience: 0,
            available_skill_points: 0,
            equipped_item:0,
            strength_stat: 23,
            endurance_stat: 10,
            agility_stat: 3,
            will_stat: 3,
            health: 10,
            mana: 0,
            stamina: 10,
            buffs: Vec::new(),
            buffs_summary: [0,0,0,0,0],
            card_id_generator: 0,
            equipment_id_generator: 0,
        };
        let buffer = char.to_bytes();
        cli_log::info!("{:?}", buffer);

        assert!(buffer.len() == HERO_ENTITY_SIZE);
    }
}