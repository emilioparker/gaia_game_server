use std::hash::Hash;

use bson::oid::ObjectId;

use crate::{ability_user::AbilityUser, buffs::buff::{Buff, BuffUser, BUFF_DEFENSE, BUFF_STRENGTH}, definitions::definitions_container::Definitions, map::tetrahedron_id::TetrahedronId};

use super::{hero_card_inventory::CardItem, hero_inventory::InventoryItem, hero_weapon_inventory::WeaponItem};

pub const HERO_ENTITY_SIZE: usize = 50;

pub const DASH_FLAG : u8 = 0b00000001;
pub const CHAT_FLAG : u8 = 0b00000010;

#[derive(Debug)]
#[derive(Clone)]
pub struct HeroEntity 
{
    pub object_id: Option<ObjectId>,
    pub player_id: Option<ObjectId>,
    pub version: u16, // 2 bytes
    pub hero_name: String,
    pub hero_id: u16, // 2 bytes
    pub faction:u8, // 1 byte
    pub position: TetrahedronId, // 6 bytes

    // 11 bytes

    pub second_position: TetrahedronId, // not sent, when saving on the database, this on is stored. On login this on is used
    pub vertex_id:i32,// not sent, also saved in db, but only used on login to properly set the position of the player.

    pub path: [u8;6], // 6 bytes
    pub time : u32,// 4 bytes // el tiempo en que inicio el recorrido.
    pub action:u8, //1 bytes

    pub flags:u8, // 1 byte

    // 12 bytes
    
    pub inventory : Vec<InventoryItem>,// this one is not serializable  normally
    pub card_inventory : Vec<CardItem>,// this one is not serializable  normally
    pub weapon_inventory : Vec<WeaponItem>,// this one is not serializable  normally
    pub inventory_version : u8, // 1 bytes

    // 1 bytes

    pub level:u8, // 1 bytes
    pub experience:u32, // 4 bytes
    pub available_skill_points:u8, // 1 bytes used for stats
    pub weapon:u8,// 1 byte

    // 6 bytes

    // attributes 4 bytes
    pub strength_points: u8, 
    pub defense_points: u8,
    pub intelligence_points: u8,
    pub mana_points: u8,

    // 4 bytes

    pub base_strength: u16,
    pub base_defense: u16,
    pub base_intelligence: u16,
    pub base_mana: u16,

    // 8 bytes

    // stats
    pub health: u16, // 2 bytes
    pub buffs : Vec<Buff>,// this one is not serializable  normally
    pub buffs_summary : [u8;5] // this one is serialized but not saved 5 bytes

    // 7 bytes 

    // 11 + 12 + 1 + 6 + 4 + 8 + 7 = 49
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
        // 5 bytes

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
        buffer[offset] = self.weapon;
        offset = end;

        end = offset + 1;
        buffer[offset] = self.strength_points;
        offset = end;

        end = offset + 1;
        buffer[offset] = self.defense_points;
        offset = end;

        end = offset + 1;
        buffer[offset] = self.intelligence_points;
        offset = end;

        end = offset + 1;
        buffer[offset] = self.mana_points;
        offset = end;

        end = offset + 2;
        let strenght_bytes = u16::to_le_bytes(self.base_strength); // 2 bytes
        buffer[offset..end].copy_from_slice(&strenght_bytes);
        offset = end;

        end = offset + 2;
        let defense_bytes = u16::to_le_bytes(self.base_defense); // 2 bytes
        buffer[offset..end].copy_from_slice(&defense_bytes);
        offset = end;

        end = offset + 2;
        let intelligence_bytes = u16::to_le_bytes(self.base_intelligence); // 2 bytes
        buffer[offset..end].copy_from_slice(&intelligence_bytes);
        offset = end;

        let mana_bytes = u16::to_le_bytes(self.base_mana); // 4 bytes
        end = offset + 2;
        buffer[offset..end].copy_from_slice(&mana_bytes);
        offset = end;

        let health_bytes = u16::to_le_bytes(self.health); // 4 bytes
        end = offset + 2;
        buffer[offset..end].copy_from_slice(&health_bytes);
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

    fn update_health(&mut self, new_health : u16, definition: &Definitions) 
    {
        self.health = new_health;
    }

    fn get_constitution(&self, definition: &Definitions) -> u16 
    {
        let character_definition = definition.character_progression.get(self.level as usize).unwrap();
        character_definition.constitution
    }
    
    fn get_total_attack(&self, card_id : u32, definition: &Definitions) -> u16 
    {
        let card_attack = definition.cards.get(card_id as usize).map_or(0f32, |d| d.strength_factor);
        let stat = HeroEntity::calculate_stat(self.base_strength, self.strength_points, 2.2f32, 1f32);
        let added_strength : f32 = self.buffs.iter().map(|b| 
            {
                if let Some(def) = definition.get_buff_by_code(b.buff_id)
                {
                    if def.buff_type == BUFF_STRENGTH
                    {
                        return def.base_value;
                    }
                }

                return 0f32;
            })
            .sum();

        let base = self.base_strength;
        let points = self.strength_points;
        cli_log::info!(" -- calculate total attack {card_attack} base {base} points {points}  stat {stat} buff {added_strength}");
        (stat as f32 * card_attack).round() as u16  + added_strength.round() as u16
    }
    
    fn get_total_defense(&self, definition: &Definitions) -> u16 
    {
        let stat = HeroEntity::calculate_stat(self.base_defense, self.defense_points, 2.2f32, 1f32);
        let added_defense : f32 = self.buffs.iter().map(|b| 
            {
                if let Some(def) = definition.get_buff_by_code(b.buff_id)
                {
                    if def.buff_type == BUFF_DEFENSE
                    {
                        return def.base_value;
                    }
                }
                return 0f32;
            })
            .sum();

        cli_log::info!("character added defense {} buffs_len: {}",added_defense, self.buffs.len());
        stat + added_defense.round() as u16
    }
}


#[cfg(test)]
mod tests 
{
    use std::num::Wrapping;


    use crate::{hero::{hero_entity::HERO_ENTITY_SIZE, hero_inventory::HERO_INVENTORY_ITEM_SIZE}, map::tetrahedron_id::TetrahedronId};

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
            action: 0,
            flags:0,
            position: TetrahedronId::default(),
            second_position: TetrahedronId::default(),
            vertex_id:-1,
            path:[0,0,0,0,0,0],
            time:0,
            inventory: Vec::new(),
            card_inventory: Vec::new(),
            weapon_inventory: Vec::new(),
            inventory_version: 1,
            health: 0,
            level: 1,
            experience: 0,
            available_skill_points: 0,
            weapon:0,
            base_strength: 0,
            base_defense: 0,
            base_intelligence: 0,
            base_mana: 0,
            strength_points: 0,
            defense_points: 0,
            intelligence_points: 0,
            mana_points: 0,
            buffs: Vec::new(),
            buffs_summary: [0,0,0,0,0],
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
            position: TetrahedronId::default(),
            second_position: TetrahedronId::default(), 
            vertex_id:-1,
            path:[0,0,0,0,0,0],
            time:0,
            action: 1,
            flags:0,
            inventory: Vec::new(),
            card_inventory: Vec::new(),
            weapon_inventory: Vec::new(),
            inventory_version: 10,
            level: 0,
            experience: 0,
            available_skill_points: 0,
            weapon:0,
            strength_points: 0,
            defense_points: 0,
            intelligence_points: 0,
            mana_points: 0,
            base_strength: 23,
            base_defense: 10,
            base_intelligence: 3,
            base_mana: 3,
            health: 10,
            buffs: Vec::new(),
            buffs_summary: [0,0,0,0,0],
        };
        let buffer = char.to_bytes();
        cli_log::info!("{:?}", buffer);

        assert!(buffer.len() == HERO_ENTITY_SIZE);
    }
}