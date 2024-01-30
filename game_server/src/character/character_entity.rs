use std::hash::Hash;

use bson::oid::ObjectId;

use crate::map::map_entity::MapEntity;

pub const CHARACTER_ENTITY_SIZE: usize = 50;
pub const CHARACTER_INVENTORY_SIZE: usize = 8;

#[derive(Debug)]
#[derive(Clone)]
pub struct CharacterEntity 
{
    pub object_id: Option<ObjectId>,
    pub player_id: Option<ObjectId>,
    pub version: u16, // 2 bytes
    pub character_name: String,
    pub character_id: u16,
    pub faction:u8,
    pub position: [f32;3],
    pub second_position: [f32;3],
    pub action:u32,
    pub inventory : Vec<InventoryItem>,// this one is not serializable  normally
    pub inventory_hash : u32,

    pub level:u8,
    // points
    pub alchemy_skill_points:u8, // used for skill tree
    pub blacksmith_skill_points:u8, // used for skill tree
    pub available_experience_points:u8, // used for stats

    // stats
    pub health: u16,

    // attributes
    pub constitution: u16,
    pub strenght: u16,
    pub dexterity: u16,
    pub intelligence: u16,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct InventoryItem
{
    pub item_id : u32, //4
    pub level : u8, //1
    pub quality : u8,//1
    pub amount : u16 // 2
}

impl InventoryItem 
{
    pub fn to_bytes(&self) -> [u8; CHARACTER_INVENTORY_SIZE]{
        let offset = 0;
        let mut buffer = [0u8;CHARACTER_INVENTORY_SIZE];
        let item_id_bytes = u32::to_le_bytes(self.item_id); // 4 bytes
        let end = offset + 4; 
        buffer[offset..end].copy_from_slice(&item_id_bytes);

        let mut offset = end;
        buffer[offset] = self.level;
        offset += 1;
        buffer[offset] = self.quality;
        offset += 1;


        let end = offset + 2; 
        let amount_bytes = u16::to_le_bytes(self.amount); // 2 bytes
        buffer[offset..end].copy_from_slice(&amount_bytes);
        buffer
    }
}


impl CharacterEntity {
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

        end = offset + 4;
        float_into_buffer(&mut buffer, self.position[0], offset, end);
        offset = end;
        end = offset + 4;
        float_into_buffer(&mut buffer, self.position[1], offset, end);
        offset = end;
        end = offset + 4;
        float_into_buffer(&mut buffer, self.position[2], offset, end);
        offset = end;

        end = offset + 4;
        float_into_buffer(&mut buffer, self.second_position[0], offset, end);
        offset = end;
        end = offset + 4;
        float_into_buffer(&mut buffer, self.second_position[1], offset, end);
        offset = end;
        end = offset + 4;
        float_into_buffer(&mut buffer, self.second_position[2], offset, end);
        offset = end;

        // 24 bytes

        let action_bytes = u32::to_le_bytes(self.action); // 4 bytes
        end = offset + 4;
        buffer[offset..end].copy_from_slice(&action_bytes);
        offset = end;
        let inventory_hash_bytes = u32::to_le_bytes(self.inventory_hash); // 4 bytes
        end = offset + 4;
        buffer[offset..end].copy_from_slice(&inventory_hash_bytes);
        offset = end;

        // 8 bytes

        end = offset + 1;
        buffer[offset] = self.level;
        offset = end;

        end = offset + 1;
        buffer[offset] = self.alchemy_skill_points;
        offset = end;

        end = offset + 1;
        buffer[offset] = self.available_experience_points;
        offset = end;

        // 3 bytes

        let health_bytes = u16::to_le_bytes(self.health); // 4 bytes
        end = offset + 2;
        buffer[offset..end].copy_from_slice(&health_bytes);
        offset = end;

        let constitution_bytes = u16::to_le_bytes(self.constitution); // 4 bytes
        end = offset + 2;
        buffer[offset..end].copy_from_slice(&constitution_bytes);
        offset = end;

        end = offset + 2;
        let strenght_bytes = u16::to_le_bytes(self.strenght); // 2 bytes
        buffer[offset..end].copy_from_slice(&strenght_bytes);
        offset = end;

        end = offset + 2;
        let dexterity_bytes = u16::to_le_bytes(self.dexterity); // 2 bytes
        buffer[offset..end].copy_from_slice(&dexterity_bytes);
        offset = end;

        end = offset + 2;
        let intelligence_bytes = u16::to_le_bytes(self.intelligence); // 2 bytes
        buffer[offset..end].copy_from_slice(&intelligence_bytes);
        offset = end;

        // 10 bytes


        //5 +24 +8 +3 + 10

        buffer
    }

    pub fn add_xp_mob_defeated(&mut self, defeated_entity : MapEntity)
    {
        
    }

    pub fn add_xp_player_defeated(&mut self, defeated_entity : MapEntity)
    {

    }

    pub fn add_inventory_item(&mut self, new_item : InventoryItem)
    {
        let mut found = false;
        for item in &mut self.inventory {
            if item.item_id == new_item.item_id && item.level == new_item.level && item.quality == new_item.quality {
                item.amount += new_item.amount;
                found = true;
            }
        }

        if !found {
            self.inventory.push(new_item);
        }

        self.inventory_hash = self.calculate_inventory_hash();
    }

    pub fn remove_inventory_item(&mut self, old_item : InventoryItem) -> bool
    {
        let mut successfuly_removed = false;
        for (index, item) in &mut self.inventory.iter_mut().enumerate() 
        {
            if item.item_id == old_item.item_id && item.level == old_item.level && item.quality == old_item.quality {
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
            self.inventory_hash = self.calculate_inventory_hash();
        }
        successfuly_removed
    }

    pub fn calculate_inventory_hash(&self) -> u32
    {
        let mut hash : u32 = 1;
        for item in &self.inventory {
            hash = hash.wrapping_mul(item.level as u32); 
            hash = hash.wrapping_mul(item.quality as u32); 
            hash = hash.wrapping_mul(item.amount as u32); 
        }
        hash
    }

}

impl Hash for CharacterEntity 
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.action.hash(state);
    }
}

fn float_into_buffer(buffer : &mut [u8], data: f32, start : usize, end: usize)
{
    let bytes = f32::to_le_bytes(data);
    buffer[start..end].copy_from_slice(&bytes);
}

#[cfg(test)]
mod tests {
    use std::num::Wrapping;

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
            position: [1.0, 2.0, 3.0],
            second_position: [1.0, 2.0, 3.0],
            inventory: Vec::new(),
            inventory_hash: 1,
            constitution: 0,
            health: 0,
            level: 1,
            alchemy_skill_points: 0,
            blacksmith_skill_points: 0,
            available_experience_points: 0,
            strenght: 0,
            dexterity: 0,
            intelligence: 0,
        };

        entity.add_inventory_item(super::InventoryItem { item_id: 1, level: 1, quality: 1, amount: 1 });
        entity.add_inventory_item(super::InventoryItem { item_id: 1, level: 1, quality: 1, amount: 2 });

        assert!(entity.inventory.len() == 1);

        let item = entity.inventory.iter().next().unwrap();
        assert!(item.amount == 3);
        entity.add_inventory_item(super::InventoryItem { item_id: 2, level: 1, quality: 1, amount: 2 });
        assert!(entity.inventory.len() == 2);
        println!("{:?}", entity.inventory);
    }

    #[test]
    fn test_encode_inventory_item()
    {

        let item = super::InventoryItem { item_id: 1, level: 1, quality: 1, amount: 1 };
        let buffer = item.to_bytes();

        assert!(buffer.len() == super::CHARACTER_INVENTORY_SIZE);
    }
}