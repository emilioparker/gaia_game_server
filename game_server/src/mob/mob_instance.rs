use crate::{ability_user::AbilityUser, buffs::buff::{self, Buff, BuffUser, BUFF_DEFENSE, BUFF_STRENGTH}, definitions::definitions_container::Definitions, map::tetrahedron_id::TetrahedronId};

pub const MOB_ENTITY_SIZE: usize = 39;

#[derive(Debug, Clone)]
pub struct MobEntity
{
    pub tile_id: TetrahedronId, // 6 bytes
    pub mob_definition_id: u16, // 2 bytes
    pub level:u8, // 1 byte
    pub version: u8, // 1 byte


    // 10 bytes

    // to handle who is commanding this tile with a timeout
    pub owner_id : u16, // 2 bytes
    pub ownership_time : u32, // 4 bytes

    //6 bytes

    // for moving between origin and target
    pub origin_id : TetrahedronId, // 6 bytes
    pub target_id : TetrahedronId, // 6 bytes
    pub time : u32,// 4 bytes

    // 16 bytes

    pub health: u16, // 2 bytes
    pub buffs : Vec<Buff>,// this one is not serializable  normally
    pub buffs_summary : [u8;5], // this one is serialized but not saved 10 bytes

    // 9 bytes

    //total 10 + 6 + 16 + 7 = 39 

}


impl MobEntity 
{
    // used by the test_client ignores the protocol byte.
    pub fn to_bytes(&self) -> [u8;MOB_ENTITY_SIZE] 
    {
        let mut buffer = [0u8;MOB_ENTITY_SIZE];
        let mut start : usize;
        let mut end : usize;
        start = 0;

        end = start + 6;
        let tile_id = self.tile_id.to_bytes(); // 6 bytes
        buffer[start..end].copy_from_slice(&tile_id);
        start = end;

        end = start + 2;
        let mob_definition_id_bytes = u16::to_le_bytes(self.mob_definition_id); // 2 bytes
        buffer[start..end].copy_from_slice(&mob_definition_id_bytes);
        start = end;

        end = start + 1;
        buffer[start] = self.level;
        start = end;

        end = start + 1;
        buffer[start] = self.version;
        start = end;

        end = start + 2;
        let owner_id_bytes = u16::to_le_bytes(self.owner_id); // 2 bytes
        buffer[start..end].copy_from_slice(&owner_id_bytes);
        start = end;

        end = start + 4;
        let ownership_bytes = u32::to_le_bytes(self.ownership_time); // 2 bytes
        buffer[start..end].copy_from_slice(&ownership_bytes);
        start = end;

        end = start + 6;
        let origin_id_bytes = self.origin_id.to_bytes(); // 6 bytes
        buffer[start..end].copy_from_slice(&origin_id_bytes);
        start = end;

        end = start + 6;
        let target_id_bytes = self.target_id.to_bytes(); // 6 bytes
        buffer[start..end].copy_from_slice(&target_id_bytes);
        start = end;

        end = start + 4;
        let time_bytes = u32::to_le_bytes(self.time); // 2 bytes
        buffer[start..end].copy_from_slice(&time_bytes);
        start = end;

        end = start + 2;
        let health_bytes = u16::to_le_bytes(self.health); // 2 bytes
        buffer[start..end].copy_from_slice(&health_bytes);
        start = end;

        // 5 pairs of 1 bytes, 10 bytes
        for buff_id in self.buffs_summary
        {
            end = start + 1;
            buffer[start] = buff_id;
            start = end;
        }

        buffer
    }

    pub fn get_size() -> usize 
    {
        MOB_ENTITY_SIZE
    }
}

impl BuffUser for MobEntity 
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

impl AbilityUser for MobEntity
{
    fn get_health(&self) -> u16 
    {
        self.health
    }

    fn get_constitution(&self, definition: &Definitions) -> u16 
    {
        let constitution = definition.mob_progression.get(self.level as usize).map_or(0, |d| d.constitution);
        constitution
    }

    fn update_health(&mut self, new_health : u16, definition: &Definitions) 
    {
        let constitution = self.get_constitution(definition);
        self.health =  new_health.min(constitution);
        println!("---- updated health {}" ,self.health)
    }
    
    fn get_total_attack(&self, card_id: u32, definition: &Definitions) -> u16 
    {
        let card_attack = definition.get_card(card_id as usize).map_or(0f32, |d| d.strength_factor);

        let (base_strength, strength_points) = definition.mob_progression.get(self.level as usize).map_or((0,0), |d| (d.base_strength, d.strength_points));
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

        let stat = MobEntity::calculate_stat(base_strength, strength_points as u8, 2.2f32, 1f32);
        (stat as f32 * card_attack).round() as u16  + added_strength.round() as u16
    }

    fn get_total_defense(&self, definition:&Definitions) -> u16
    {
        let (base_defense, defense_points) = definition.mob_progression.get(self.level as usize).map_or((0,0), |d| (d.base_defense, d.defense_points));
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

        let stat = MobEntity::calculate_stat(base_defense, defense_points as u8, 2.2f32, 1f32);
        let level = self.level;
        println!(" -- for level {level} calculate total defense base {base_defense} points {defense_points}  stat {stat} buff {added_defense}");
        stat + added_defense.round() as u16
    }
}