use std::hash::Hash;

use bson::oid::ObjectId;

use crate::{definitions::definitions_container::Definitions, map::{map_entity::MapEntity, tetrahedron_id::TetrahedronId}};

pub const CHARACTER_ENTITY_SIZE: usize = 56;
pub const CHARACTER_INVENTORY_SIZE: usize = 7;

pub const ITEMS_PRIME_KEYS: [u16;46] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193, 197, 199]; 

#[derive(Debug)]
#[derive(Clone)]
pub struct CharacterEntity 
{
    pub object_id: Option<ObjectId>,
    pub player_id: Option<ObjectId>,
    pub version: u16, // 2 bytes
    pub character_name: String,
    pub character_id: u16, // 2 bytes
    pub faction:u8, // 1 byte
    pub position: TetrahedronId, // 6 bytes
    pub second_position: TetrahedronId, // 6 bytes
    pub time : u32,// 4 bytes // el tiempo en que inicio el recorrido.
    pub action:u8, //4 bytes
    pub inventory : Vec<InventoryItem>,// this one is not serializable  normally
    pub inventory_version : u32, // 4 bytes

    // total = 25 bytes

    pub level:u8, // 1 bytes
    pub experience:u32, // 4 bytes
    pub available_skill_points:u8, // 1 bytes used for stats

    // attributes 4 bytes
    pub strength_points: u8, 
    pub defense_points: u8,
    pub intelligence_points: u8,
    pub mana_points: u8,

    // attributes 8 bytes
    pub base_strength: u16,
    pub base_defense: u16,
    pub base_intelligence: u16,
    pub base_mana: u16,

    // total 18

    // stats
    pub health: u16, // 2 bytes
    pub buffs : Vec<Buff>,// this one is not serializable  normally
    pub buffs_summary : [(u8,u8);5] // this one is serialized but not saved 10 bytes

    // total 12 bytes

    // 25 + 18 + 12
}

pub enum ItemType
{
    Material = 0,
    Card = 1,
    Equipment = 2
}

#[derive(Debug)]
#[derive(Clone)]
pub struct InventoryItem
{
    pub item_id : u32, //4
    pub equipped : u8, // 1 // this can be used to know where it is equipped. 0 means not equipped, 1 means equipped.
    pub amount : u16 // 2
}

impl InventoryItem 
{
    pub fn to_bytes(&self) -> [u8; CHARACTER_INVENTORY_SIZE]{
        let mut start = 0;
        let mut buffer = [0u8;CHARACTER_INVENTORY_SIZE];
        let item_id_bytes = u32::to_le_bytes(self.item_id); // 4 bytes
        let end = start + 4; 
        buffer[start..end].copy_from_slice(&item_id_bytes);
        start = end;

        buffer[start] = self.equipped;
        start += 1;

        let end = start + 2; 
        let amount_bytes = u16::to_le_bytes(self.amount); // 2 bytes
        buffer[start..end].copy_from_slice(&amount_bytes);
        buffer
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Stat
{
    Strength,
    Defense,
    Intelligence,
    Mana
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Buff
{
    pub card_id : u32, //1
    pub stat : Stat, //1
    pub buff_amount : f32, // 4
    pub hits: u8,// 1
    pub expiration_time:u32 //4
}

impl Stat
{
    pub fn to_byte(&self) -> u8
    {
        let stat = match self
        {
            Stat::Strength => 0,
            Stat::Defense => 1,
            Stat::Intelligence => 2,
            Stat::Mana => 3,
        };

        stat as u8
    }

    pub fn from_byte(data :u8) -> Stat
    {
        let stat = match data
        {
            0 => Stat::Strength,
            1 => Stat::Defense,
            2 => Stat::Intelligence,
            3 => Stat::Mana,
            _ => Stat::Mana,
        };
        stat
    }
}


impl CharacterEntity 
{
    pub fn to_bytes(&self) -> [u8;CHARACTER_ENTITY_SIZE] {
        let mut buffer = [0u8; CHARACTER_ENTITY_SIZE];
        let mut offset = 0;
        let mut end;

        end = offset + 2;
        let player_id_bytes = u16::to_le_bytes(self.character_id); // 2 bytes
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

        end = offset + 6;
        let target_position_tile_id_bytes = self.second_position.to_bytes();
        buffer[offset..end].copy_from_slice(&target_position_tile_id_bytes);
        offset = end;

        end = offset + 4;
        let time_bytes = u32::to_le_bytes(self.time); // 4 bytes
        buffer[offset..end].copy_from_slice(&time_bytes);
        offset = end;

        // 16 bytes

        end = offset + 1;
        buffer[offset] = self.action;

        offset = end;
        let inventory_version_bytes = u32::to_le_bytes(self.inventory_version); // 4 bytes
        end = offset + 4;
        buffer[offset..end].copy_from_slice(&inventory_version_bytes);
        offset = end;

        // 5 bytes

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

        // 20

        // 5 pairs of 1 bytes, 10 bytes
        for pair in self.buffs_summary
        {
            end = offset + 1;
            buffer[offset] = pair.0;
            offset = end;
            
            end = offset + 1;
            buffer[offset] = pair.1;
            offset = end;
        }

        buffer
    }

    pub fn add_buff(&mut self, card_id:u32, definitions: &Definitions) -> bool
    {
        if self.has_buff(card_id) 
        {
            return false;
        }

        if let Some(card) = definitions.get_card(card_id as usize)
        {
            if card.card_type == "passive"
            {
                if card.defense_factor > 0f32
                {
                    self.buffs.push(Buff
                    {
                        card_id,
                        stat: Stat::Defense,
                        buff_amount: card.defense_factor,
                        hits: card.hits,
                        expiration_time: 100,
                    })
                }
                if card.strength_factor > 0f32
                {
                    self.buffs.push(Buff
                    {
                        card_id,
                        stat: Stat::Strength,
                        buff_amount: card.strength_factor,
                        hits: card.hits,
                        expiration_time: 100,
                    })
                }
            }
            self.version += 1;
            self.summarize_buffs();
        }

        return true;
    }

    pub fn use_buffs(&mut self, used_stats : Vec<Stat>)
    {
        self.buffs.iter_mut()
        .filter(|b| used_stats.contains(&b.stat))
        .for_each(|b| b.hits = b.hits.saturating_sub(1));
        let updated_buffs : Vec<Buff> = self.buffs.iter().filter(|b| b.hits > 0).map(|b| b.clone()).collect();
        self.buffs = updated_buffs;
        self.summarize_buffs();
        self.version += 1;
    }

    pub fn has_buff(&self, card_id : u32) -> bool
    {
        let mut found = false;
        for buff in &self.buffs 
        {
            if buff.card_id == card_id
            {
                found = true;
            }
        }
        return found;
    }

    pub fn summarize_buffs(&mut self)
    {
        // let mut buffs_summary : [(u8,u8);5]= [(0, 0),(0, 0), (0, 0), (0, 0), (0, 0)];
        let mut index = 0;
        for value in self.buffs_summary.iter_mut()
        {
            if let Some(buff) = self.buffs.get(index)
            {
                *value = ((buff.card_id - 10000) as u8, buff.hits);//(buff.card_id, buff.hits);
            }
            else
            {
                *value = (0u8,0u8);//(buff.card_id, buff.hits);
            }
            index += 1;
        }
    }

    pub fn add_xp_mob_defeated(&mut self, definitions: &Definitions)
    {
        self.experience += 1;
        if let Some(next_level_data) = definitions.character_progression.get(self.level as usize + 1)
        {
            if next_level_data.required_xp <= self.experience
            {
                self.level += 1;
                self.available_skill_points = self.available_skill_points.wrapping_add(next_level_data.skill_points as u8);
            }
        }
        println!("----- add xp mob defeated {}", self.experience);
    }

    pub fn add_xp_player_defeated(&mut self, _defeated_entity : MapEntity)
    {

    }

    pub fn has_inventory_item(&self, id : u32) -> bool
    {
        let mut found = false;
        for item in &self.inventory 
        {
            if item.item_id == id
            {
                found = true;
            }
        }
        return found;
    }

    pub fn add_inventory_item(&mut self, new_item : InventoryItem)
    {
        let mut found = false;
        for item in &mut self.inventory 
        {
            if item.item_id == new_item.item_id && item.equipped == new_item.equipped 
            {
                item.amount += new_item.amount;
                found = true;
            }
        }

        if !found 
        {
            self.inventory.push(new_item);
            self.version += 1;
        }

        self.inventory_version += 1;
    }

    pub fn remove_inventory_item(&mut self, old_item : InventoryItem) -> bool
    {
        let mut successfuly_removed = false;
        for (index, item) in &mut self.inventory.iter_mut().enumerate() 
        {
            if item.item_id == old_item.item_id && item.equipped == old_item.equipped
            {
                if item.amount >= old_item.amount
                {
                    item.amount -= old_item.amount;
                    successfuly_removed = true;
                }

                if item.amount == 0 
                {
                    self.inventory.swap_remove(index);
                }
                break;
            }
        }

        if successfuly_removed {
            self.inventory_version += 1;
            self.version += 1;
        }
        successfuly_removed
    }

    pub fn count_items_in_slot(&mut self, slot:u8) -> usize
    {
        self.inventory.iter().filter(|i| i.equipped == slot).count()
    }

    pub fn equip_inventory_item(&mut self, item_id : u32, current_slot : u8, slot: u8) -> bool
    {
        let count = self.count_items_in_slot(slot);
        if slot == 1 && count >= 10
        {
            return false;
        }

        let mut successfuly_removed = false;
        for (index, item) in &mut self.inventory.iter_mut().enumerate() 
        {
            if item.item_id == item_id && item.equipped == current_slot
            {
                if item.amount > 0
                {
                    item.amount -= 1;
                    successfuly_removed = true;
                }

                if item.amount == 0 
                {
                    self.inventory.swap_remove(index);
                }
                break;
            }
        }


        if successfuly_removed 
        {
            self.add_inventory_item(InventoryItem { item_id, equipped: slot, amount: 1 });
            self.inventory_version += 1;
            self.version += 1;
        }
        successfuly_removed
    }

    // pub fn calculate_inventory_hash(&self) -> u32
    // {
    //     let mut hash : u32 = 0;
    //     let mut index = 1;
    //     for item in &self.inventory 
    //     {
    //         let salt = ITEMS_PRIME_KEYS[index] as u32;
    //         let key = (item.item_id + 1).wrapping_mul(salt);
    //         let pair = key.wrapping_mul(item.amount as u32);
    //         hash = hash.wrapping_add(pair); 
    //         index += 1;
    //     }
    //     println!("hash {hash}");
    //     hash
    // }

    pub fn get_strength(&self, strength : f32) -> u16
    {
        let stat = CharacterEntity::calculate_stat(self.base_strength, self.strength_points, 2.2f32, 1f32);
        let added_strength : f32 = self.buffs.iter().filter(|b| b.stat == Stat::Strength).map(|b| b.buff_amount).sum();
        (stat as f32 * strength).round() as u16  + added_strength.round() as u16
    }

    pub fn get_defense(&self, defense :f32) -> u16
    {
        let stat = CharacterEntity::calculate_stat(self.base_defense, self.defense_points, 2.2f32, 1f32);
        let added_defense : f32 = self.buffs.iter().filter(|b| b.stat == Stat::Defense).map(|b| b.buff_amount).sum();
        (stat as f32 * defense).round() as u16  + added_defense.round() as u16
    }

    pub fn calculate_stat(base : u16, points : u8, class_multiplier:f32, efficiency:f32) -> u16
    {
        (base as f32 + (points as f32) * class_multiplier * efficiency).round() as u16
    }

}

impl Hash for CharacterEntity 
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.action.hash(state);
    }
}


#[cfg(test)]
mod tests {
    use std::num::Wrapping;


    use crate::{character::character_entity::CHARACTER_ENTITY_SIZE, map::tetrahedron_id::TetrahedronId};

    use super::CharacterEntity;


    #[test]
    fn test_enconde_ascii() {
        // いいえ
        let mut ch:char='い';
    
        println!("ASCII value: {}",ch as u32);
        
        ch='&';
        println!("ASCII value: {}",ch as u32);

        ch='X';
        println!("ASCII value: {}",ch as u32); 
    }

    #[test]
    fn test_convert_string_to_array() {
        let name = "aaaa".to_string();
        let filled = format!("{: <5}", name);
        println!("filled {}", filled);
        let name_data : Vec<u32> = filled.chars().into_iter().map(|c| c as u32).collect();

        let mut name_array = [0u32; 5];
        name_array.clone_from_slice(&name_data.as_slice()[0..5]);
        println!("{:?}", name_array);
    }

    #[test]
    fn test_overflow()
    {
        let a = Wrapping(200u8);
        let b = Wrapping(2u8);
        let c = Wrapping(121u8);
        let d = Wrapping(15u8);
        let result = a * b * c * d;
        println!("{result}");
        let result = c * b * d * a;
        println!("{result}");
        let result = a * c * d * b;
        println!("{result}");
    }

    #[test]
    fn test_add_inventory_item()
    {
        let mut entity = CharacterEntity{
            object_id: None,
            player_id: None,
            version:1,
            character_name: "a".to_owned(),
            character_id: 1234,
            faction:0,
            action: 0,
            position: TetrahedronId::from_string("A"),
            second_position: TetrahedronId::from_string("A"),
            time:0,
            inventory: Vec::new(),
            inventory_version: 1,
            health: 0,
            level: 1,
            experience: 0,
            available_skill_points: 0,
            base_strength: 0,
            base_defense: 0,
            base_intelligence: 0,
            base_mana: 0,
            strength_points: 0,
            defense_points: 0,
            intelligence_points: 0,
            mana_points: 0,
            buffs: Vec::new(),
            buffs_summary: [(0,0),(0,0),(0,0),(0,0),(0,0)],
        };

        entity.add_inventory_item(super::InventoryItem { item_id: 1, equipped: 0, amount: 1 });
        entity.add_inventory_item(super::InventoryItem { item_id: 1, equipped: 0, amount: 2 });

        assert!(entity.inventory.len() == 1);

        let item = entity.inventory.iter().next().unwrap();
        assert!(item.amount == 3);
        entity.add_inventory_item(super::InventoryItem { item_id: 2, equipped: 1, amount: 2 });
        assert!(entity.inventory.len() == 2);
        println!("{:?}", entity.inventory);
    }

    #[test]
    fn test_encode_inventory_item()
    {

        let item = super::InventoryItem { item_id: 1, equipped: 1, amount: 1 };
        let buffer = item.to_bytes();

        assert!(buffer.len() == super::CHARACTER_INVENTORY_SIZE);
    }

    #[test]
    fn test_encode_character()
    {

        let char = CharacterEntity{
            object_id: None,
            player_id: None,
            version: 1,
            character_name: "Park".to_string(),
            character_id: 2,
            faction: 0,
            position: TetrahedronId::from_string("A"),
            second_position: TetrahedronId::from_string("A"),
            time:0,
            action: 1,
            inventory: Vec::new(),
            inventory_version: 10,
            level: 0,
            experience: 0,
            available_skill_points: 0,
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
            buffs_summary: [(0,0),(0,0),(0,0),(0,0),(0,0)],
        };
        let buffer = char.to_bytes();
        println!("{:?}", buffer);

        assert!(buffer.len() == CHARACTER_ENTITY_SIZE);
    }
}