use std::hash::Hash;
use std::sync::atomic::AtomicU32;

use bson::oid::ObjectId;
use rand::rngs::StdRng;

use crate::ability_user::AbilityUser;
use crate::buffs::buff::Buff;
use crate::buffs::buff::BuffUser;
use crate::definitions::definitions_container::Definitions;
use crate::map::tetrahedron_id::TetrahedronId;

use super::hero_card_inventory::CardItem;
use super::hero_equipment_inventory::EquipmentItem;
use super::hero_inventory::InventoryItem;
use super::hero_skill_inventory::SkillData;

pub const HERO_ENTITY_SIZE: usize = 63;

pub const DASH_FLAG : u8 = 0b00000001;
pub const CHAT_FLAG : u8 = 0b00000010;


pub struct InventoryType(pub u8);

impl InventoryType 
{
    pub const ITEMS :      u8 = 0;
    pub const CARDS:       u8 = 1;
    pub const EQUIPMENT:   u8 = 2;
}

#[derive(Debug)]
pub struct HeroEntity
{
    pub object_id: Option<ObjectId>,
    pub player_id: Option<ObjectId>,
    pub version: u16, // 2 bytes
    pub hero_name: String, // not serializable
    pub hero_id: u16, // 2 bytes
    pub faction:u8, // 1 byte
    pub profession:u8, // 1 byte
    pub realm:u8, // 1 byte

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

    pub armor : u16,
    pub right_weapon : u16,
    pub left_weapon : u16,

    pub level:u8, // 1 bytes
    pub experience:u32, // 4 bytes
    pub available_skill_points:u8, // 1 bytes used for stats

    // 7 bytes

    pub strength_stat: u16,
    pub vitality_stat: u16,
    pub agility_stat: u16,
    pub will_stat: u16,
    pub regeneration_time: u32, // 4 bytes

    // 12 bytes

    // stats
    pub health: u16, // 2 bytes
    pub mana: u16, // 2 bytes
    pub stamina: u16, // 2 bytes
    pub intelligence: u16, // 2 bytes


    pub buffs : Vec<Buff>,// this one is not serializable  normally
    pub buffs_summary : [u8;5], // this one is serialized but not saved 5 bytes

    pub card_id_generator : u32, // not serializable
    pub equipment_id_generator : u32, // not serializable

    // 11 bytes

    // 13 + 12 + 7 + 8 + 14 = 54
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
            realm: self.realm,
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
            armor: self.armor,
            right_weapon: self.right_weapon,
            left_weapon: self.left_weapon,
            level: self.level,
            experience: self.experience,
            available_skill_points: self.available_skill_points,
            strength_stat: self.strength_stat,
            vitality_stat: self.vitality_stat,
            agility_stat: self.agility_stat,
            will_stat: self.will_stat,
            regeneration_time: self.regeneration_time,
            health: self.health,
            mana: self.mana,
            stamina: self.stamina,
            intelligence: self.intelligence,
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
            realm: self.realm,
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
            armor: self.armor,
            right_weapon: self.right_weapon,
            left_weapon: self.left_weapon,
            level: self.level,
            experience: self.experience,
            available_skill_points: self.available_skill_points,
            strength_stat: self.strength_stat,
            vitality_stat: self.vitality_stat,
            agility_stat: self.agility_stat,
            will_stat: self.will_stat,
            regeneration_time: self.regeneration_time,
            health: self.health,
            mana: self.mana,
            stamina: self.stamina,
            intelligence: self.intelligence,
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

        end = offset + 1;
        buffer[offset] = self.realm;
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

        end = offset + 2;
        let armor_bytes = u16::to_le_bytes(self.armor);
        buffer[offset..end].copy_from_slice(&armor_bytes);
        offset = end;

        end = offset + 2;
        let right_weapon_bytes = u16::to_le_bytes(self.right_weapon);
        buffer[offset..end].copy_from_slice(&right_weapon_bytes);
        offset = end;

        end = offset + 2;
        let left_weapon_bytes = u16::to_le_bytes(self.left_weapon);
        buffer[offset..end].copy_from_slice(&left_weapon_bytes);
        offset = end;

        // 6 bytes

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

        end = offset + 2;
        let strength_bytes = u16::to_le_bytes(self.strength_stat); // 2 bytes
        buffer[offset..end].copy_from_slice(&strength_bytes);
        offset = end;

        end = offset + 2;
        let vitality_bytes = u16::to_le_bytes(self.vitality_stat); // 2 bytes
        buffer[offset..end].copy_from_slice(&vitality_bytes);
        offset = end;

        end = offset + 2;
        let agility_bytes = u16::to_le_bytes(self.agility_stat); // 2 bytes
        buffer[offset..end].copy_from_slice(&agility_bytes);
        offset = end;

        let will_bytes = u16::to_le_bytes(self.will_stat); // 2 bytes
        end = offset + 2;
        buffer[offset..end].copy_from_slice(&will_bytes);
        offset = end;

        let regeneration_time_bytes = u32::to_le_bytes(self.regeneration_time); // 4 bytes
        end = offset + 4;
        buffer[offset..end].copy_from_slice(&regeneration_time_bytes);
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

        let intelligence_bytes = u16::to_le_bytes(self.intelligence); // 2 bytes
        end = offset + 2;
        buffer[offset..end].copy_from_slice(&intelligence_bytes);
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

                let mut random_generator = <StdRng as rand::SeedableRng>::from_entropy();
                let profession = definitions.professions_by_id.get(self.profession as usize);

                let potentials = [
                    (self.strength_stat,  profession.map_or(0, |p| p.strength.max)),
                    (self.vitality_stat, profession.map_or(0, |p| p.vitality.max)),
                    (self.agility_stat,   profession.map_or(0, |p| p.agility.max)),
                    (self.will_stat,      profession.map_or(0, |p| p.will.max)),
                ];

                let gains: [u16; 4] = potentials.map(|(current, potential)| {
                    let roll = (rand::Rng::gen::<f32>(&mut random_generator) * 100.0).floor() as u8;
                    let difference = potential.saturating_sub(current);
                    definitions.stat_gains.get(difference as usize).map_or(0, |g| g.get_gain(roll) as u16)
                });

                self.strength_stat  += gains[0];
                self.vitality_stat += gains[1];
                self.agility_stat   += gains[2];
                self.will_stat      += gains[3];
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

    pub fn max_stats(&mut self, definitions: &Definitions)
    {
        self.health = self.get_max_hp(definitions);
        self.mana = self.get_max_mana(definitions);
        self.stamina = self.get_max_stamina(definitions);
        self.intelligence = self.get_max_intelligence(definitions);
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
    
    fn get_stats(&self) -> (u16, u16, u16, u16)
    {
        (self.strength_stat, self.vitality_stat, self.agility_stat, self.will_stat)
    }

    // I keep this here, because this might be affected by other things.. like buffs ? Not only that, I need mobs to use the same battle code.
    fn get_hp(&self) -> u16 
    {
        self.health
    }

    fn get_mana(&self) -> u16 
    {
        self.mana
    }
    
    fn get_stamina(&self) -> u16 
    {
        self.stamina
    }
    
    fn get_intelligence(&self) -> u16 
    {
        self.intelligence
    }
    
    fn get_max_mana(&self, definition: &Definitions) -> u16 
    {
        let multiplier = definition.realms.get(self.realm as usize)
            .map_or(1f32, |d| d.multiplier);

        let result = (self.will_stat as f32 * 0.8f32 + self.vitality_stat as f32 * 0.2f32) * multiplier;
        result.round() as u16
    }
    
    fn get_max_intelligence(&self, definition: &Definitions) -> u16 
    {
        let multiplier = definition.realms.get(self.realm as usize)
            .map_or(1f32, |d| d.multiplier);

        let result = self.will_stat as f32 * multiplier;
        result.round() as u16
    }
    
    fn get_max_stamina(&self, definition: &Definitions) -> u16 
    {
        let multiplier = definition.realms.get(self.realm as usize)
            .map_or(1f32, |d| d.multiplier);

        let result = (self.vitality_stat as f32 * 0.8f32 + self.strength_stat as f32 * 0.2f32) * multiplier;
        result.round() as u16
    }
    
    fn get_max_hp(&self, definition: &Definitions) -> u16 
    {
        let multiplier = definition.realms.get(self.realm as usize)
            .map_or(1f32, |d| d.multiplier);

        let result = self.vitality_stat as f32 * multiplier;
        result.round() as u16
    }
}


// #[cfg(test)]
// mod battle_tests;

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
            realm:0,
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
            armor: 0,
            right_weapon: 0,
            left_weapon: 0,
            health: 0,
            mana: 0,
            level: 1,
            experience: 0,
            available_skill_points: 0,
            strength_stat: 0,
            vitality_stat: 0,
            agility_stat: 0,
            will_stat: 0,
            regeneration_time: 0,
            buffs: Vec::new(),
            buffs_summary: [0,0,0,0,0],
            stamina: 0,
            intelligence: 0,
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
            realm: 0,
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
            armor: 0,
            right_weapon: 0,
            left_weapon: 0,
            level: 0,
            experience: 0,
            available_skill_points: 0,
            strength_stat: 23,
            vitality_stat: 10,
            agility_stat: 3,
            will_stat: 3,
            regeneration_time: 0,
            health: 10,
            mana: 0,
            stamina: 10,
            intelligence: 0,
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