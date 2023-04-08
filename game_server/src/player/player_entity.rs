use std::{hash::Hash, collections::HashMap};

use bson::oid::ObjectId;

pub const PLAYER_ENTITY_SIZE: usize = 48;
pub const PLAYER_INVENTORY_SIZE: usize = 8;

#[derive(Debug)]
#[derive(Clone)]
pub struct PlayerEntity {
    pub object_id: Option<ObjectId>,
    pub character_name: String,
    pub player_id: u64,
    pub position: [f32;3],
    pub second_position: [f32;3],
    pub action:u32,
    pub inventory : Vec<InventoryItem>,// this one is not serializable  normally
    pub inventory_hash : u32,
    pub constitution: u32,
    pub health: u32
}

#[derive(Debug)]
#[derive(Clone)]
pub struct InventoryItem{
    pub item_id : u32, //4
    pub level : u8, //1
    pub quality : u8,//1
    pub amount : u16 // 2
}

impl InventoryItem {
    pub fn to_bytes(&self) -> [u8; PLAYER_INVENTORY_SIZE]{
        let offset = 0;
        let mut buffer = [0u8;PLAYER_INVENTORY_SIZE];
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


// #[derive(Debug)]
// #[derive(Clone)]
// pub struct PlayerInventory{
//     pub items : Vec<InventoryItem>,
//     pub hash : u32
// }

impl PlayerEntity {
    pub fn to_bytes(&self) -> [u8;PLAYER_ENTITY_SIZE] {
        let mut buffer = [0u8; PLAYER_ENTITY_SIZE];

        let player_id_bytes = u64::to_le_bytes(self.player_id); // 8 bytes
        buffer[..8].copy_from_slice(&player_id_bytes);

        float_into_buffer(&mut buffer, self.position[0], 8, 12);
        float_into_buffer(&mut buffer, self.position[1], 12, 16);
        float_into_buffer(&mut buffer, self.position[2], 16, 20);

        float_into_buffer(&mut buffer, self.second_position[0], 20, 24);
        float_into_buffer(&mut buffer, self.second_position[1], 24, 28);
        float_into_buffer(&mut buffer, self.second_position[2], 28, 32);
        let action_bytes = u32::to_le_bytes(self.action); // 4 bytes
        buffer[32..36].copy_from_slice(&action_bytes);
        let inventory_hash_bytes = u32::to_le_bytes(self.inventory_hash); // 4 bytes
        buffer[36..40].copy_from_slice(&inventory_hash_bytes);
        let constitution_bytes = u32::to_le_bytes(self.constitution); // 4 bytes
        buffer[40..44].copy_from_slice(&constitution_bytes);
        let health_bytes = u32::to_le_bytes(self.health); // 4 bytes
        buffer[44..48].copy_from_slice(&health_bytes);
        buffer
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

impl Hash for PlayerEntity {
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

    use super::PlayerEntity;


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
        let mut entity = PlayerEntity{
            object_id: None,
            character_name: "a".to_owned(),
            player_id: 1234,
            action: 0,
            position: [1.0, 2.0, 3.0],
            second_position: [1.0, 2.0, 3.0],
            inventory: Vec::new(),
            inventory_hash: 1,
            constitution: 0,
            health: 0,
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

        assert!(buffer.len() == super::PLAYER_INVENTORY_SIZE);
    }
}