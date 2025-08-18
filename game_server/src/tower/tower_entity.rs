use std::{cmp, time::SystemTime};

use bson::oid::ObjectId;

use crate::map::tetrahedron_id::{self, TetrahedronId};

pub const TOWER_ENTITY_SIZE: usize = 65;
pub const TOWER_DAMAGE_RECORD_SIZE: usize = 5;

#[derive(Debug)]
#[derive(Clone)]
pub struct TowerEntity 
{
    pub object_id: Option<ObjectId>,
    pub version: u16, // 2 bytes
    pub tetrahedron_id : TetrahedronId, // 6 bytes
    pub event_id:u16, // 2 
    pub faction:u8, // 1
    pub damage_received_in_event : Vec<DamageByFaction>,// this one is not serializable  normally
}

#[derive(Debug)]
#[derive(Clone)]
pub struct DamageByFaction
{
    pub event_id : u16, // 2
    pub faction : u8, //1
    pub amount : u16, //2
}

impl DamageByFaction 
{
    pub fn to_bytes(&self, offset : &mut usize, buffer : &mut [u8; TOWER_ENTITY_SIZE])
    {
        // let mut offset = 0;
        // let mut buffer = [0u8;TOWER_DAMAGE_RECORD_SIZE];
        let mut local_offset = *offset;

        let event_id_bytes = u16::to_le_bytes(self.event_id); // 2 bytes
        let end = local_offset + 2; 
        buffer[local_offset..end].copy_from_slice(&event_id_bytes);
        local_offset = end;

        buffer[local_offset] = self.faction; //1 byte
        local_offset += 1;


        let damage_amount_bytes = u16::to_le_bytes(self.amount); // 2 bytes
        let end = local_offset + 2; 
        buffer[local_offset..end].copy_from_slice(&damage_amount_bytes);
        local_offset = end;

        *offset = local_offset;
    }
}


impl TowerEntity 
{
    pub fn to_bytes(&self) -> [u8;TOWER_ENTITY_SIZE] {
        let mut buffer = [0u8; TOWER_ENTITY_SIZE];
        let mut offset = 0;
        let mut end;

        end = offset + 2;
        let version_bytes = u16::to_le_bytes(self.version); // 2 bytes
        buffer[..end].copy_from_slice(&version_bytes);
        offset = end;

        end = offset + 6;
        let tile_id = self.tetrahedron_id.to_bytes(); // 6 bytes
        buffer[offset..end].copy_from_slice(&tile_id);
        offset = end;

        // end = offset + 4;
        // let cooldown_bytes = u32::to_le_bytes(self.cooldown); // 2 bytes
        // buffer[offset..end].copy_from_slice(&cooldown_bytes);
        // offset = end;

        end = offset + 2;
        let event_id_bytes = u16::to_le_bytes(self.event_id); // 2 bytes
        buffer[offset..end].copy_from_slice(&event_id_bytes);
        offset = end;

        end = offset + 1;
        buffer[offset] = self.faction;
        offset = end;

        let mut count = 0;
        for item in &self.damage_received_in_event 
        {
            if item.event_id == self.event_id && count < 10
            {
                item.to_bytes(&mut offset, &mut buffer);
                count += 1;
            }
        }

        let padding = 10 - count;

        let empty_damage_record = DamageByFaction 
        {
            event_id: 0,
            faction: 0,
            amount: 0,
        };

        // fill in extra items to get to the right amount.
        for _ in 0..padding
        {
            empty_damage_record.to_bytes(&mut offset, &mut buffer);
        }

        buffer
    }

    pub fn add_damage_record(&mut self, faction : u8, event_id:u16, amount : u16) -> u16
    {
        if self.faction == faction 
        {
            return 0;
        }

        for item in &mut self.damage_received_in_event 
        {
            if item.faction == faction && item.event_id == event_id
            {
                item.amount = item.amount.saturating_add(amount);
                // it should be only one item
                return item.amount;
            }
        }

        self.damage_received_in_event.push(DamageByFaction { event_id, faction, amount});
        return amount;
    }

    pub fn repair_damage(&mut self, faction : u8, event_id:u16, amount : u16)
    {
        if self.faction != faction 
        {
            return;
        }

        for item in &mut self.damage_received_in_event 
        {
            if item.faction != faction && item.event_id == event_id
            {
                item.amount = item.amount.saturating_sub(amount);
            }
        }
    }

    pub fn get_damage_by_faction(&self, faction : u8) -> u16
    {
        let mut total_damage : u16 = 0;
        for item in &self.damage_received_in_event 
        {
            if item.event_id == self.event_id && item.faction == faction
            {
                total_damage = total_damage.saturating_add(item.amount);
            }
        }
        total_damage
    }

    pub fn calculate_total_damage(&mut self) -> u16
    {
        let mut total_damage : u16 = 0;
        for item in &self.damage_received_in_event 
        {
            if item.event_id == self.event_id
            {
                total_damage = total_damage.saturating_add(item.amount);        
            }
        }

        total_damage
    }

    pub fn remove_old_event_entries(&mut self)
    {
        self.damage_received_in_event.retain(|r| r.event_id == self.event_id);
    }

    pub fn finish_event(&mut self)
    {
        self.remove_old_event_entries(); // this will remove old event entries.
        let winner = self.damage_received_in_event.iter().max_by(|a, b| a.amount.cmp(&b.amount));
        if let Some(winner) = winner 
        {
            self.faction = winner.faction;
        }
        self.event_id += 1;
        self.damage_received_in_event.clear();
    }

    pub fn is_active(&self, faction : u8, current_time:u32) -> bool
    {
        TowerEntity::is_tower_active(&self.tetrahedron_id, self.faction, faction, current_time)
    }

    pub fn is_tower_active(tile_id : &TetrahedronId, tower_faction : u8, faction : u8, current_time:u32) -> bool
    {
        // tower has been conquered and can be used by the faction
        if tower_faction == faction
        {
            return false;
        }
        else
        {
            let active_time = 5;
            let inactive_time = 1;
            let total_cycle = active_time + inactive_time;
            let elapsed_time = cmp::max(0, current_time + (tile_id.id + tile_id.area as u32) * 10) % (total_cycle*60);
            if elapsed_time <= inactive_time * 60
            {
                return false;
            }
            else 
            {
                return true;
            }
        }
    } 

    pub fn get_size() -> usize 
    {
        TOWER_ENTITY_SIZE
    }
}

#[cfg(test)]
mod tests {

    use crate::{tower::tower_entity::TOWER_ENTITY_SIZE, map::tetrahedron_id::TetrahedronId};

    use super::TowerEntity;

    #[test]
    fn test_encode_decode_tower()
    {
        let mut tower_entity = TowerEntity 
        {
            object_id: None,
            tetrahedron_id: TetrahedronId::from_string("a0"),
            version: 0,
            event_id: 1,
            faction: 0,
            damage_received_in_event: Vec::new()
        };

        tower_entity.add_damage_record(0, 1, 10);
        tower_entity.add_damage_record(1, 1, 12);
        tower_entity.add_damage_record(1, 1, 5);

        let data = tower_entity.to_bytes();
        assert_eq!(data.len(), TOWER_ENTITY_SIZE);
    }

    #[test]
    fn test_add_damage_record()
    {
        let mut entity = TowerEntity
        {
            object_id: None,
            tetrahedron_id: TetrahedronId::from_string("a0"),
            version: 0,
            event_id: 0,
            faction: 0,
            damage_received_in_event: Vec::new(),
        };

        entity.add_damage_record(0, 0, 10);
        entity.add_damage_record(1, 0, 12);
        entity.add_damage_record(1, 0, 5);

        assert!(entity.damage_received_in_event.len() == 2);

        let damage = entity.get_damage_by_faction(1);
        assert_eq!(damage, 17);

        //add more damage
        entity.add_damage_record(1, 0, 9);
        let damage = entity.get_damage_by_faction(1);
        assert_eq!(damage, 26);

        cli_log::info!("{:?}", entity.damage_received_in_event);
    }
}