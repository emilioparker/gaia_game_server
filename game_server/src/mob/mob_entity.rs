use rand::rngs::StdRng;

use crate::ability_user::AbilityUser;
use crate::buffs::buff::Buff;
use crate::buffs::buff::BuffUser;
use crate::definitions::definitions_container::Definitions;
use crate::map::tetrahedron_id::TetrahedronId;

pub const MOB_ENTITY_SIZE: usize = 37;

#[derive(Debug, Clone)]
pub struct MobEntity
{
    pub mob_id: u32, // 4 bytes
    pub mob_definition_id: u16, // 2 bytes
    pub level:u8, // 1 byte
    pub version: u8, // 1 byte


    // 8 bytes

    // to handle who is commanding this tile with a timeout
    pub owner_id : u16, // 2 bytes
    pub ownership_time : u32, // 4 bytes

    //6 bytes

    // for moving between origin and target
    pub start_position_id: TetrahedronId, // 6 bytes
    pub end_position_id: TetrahedronId, // 6 bytes not serialized, path should point to this point
    pub path: [u8;6], // 6 bytes
    pub time : u32,// 4 bytes // el tiempo en que inicio el recorrido.

    // 16 bytes

    pub strength_stat: u16,
    pub vitality_stat: u16,
    pub agility_stat: u16,
    pub will_stat: u16,

    // 8 bytes

    // stats
    pub health: u16, // 2 bytes

    pub buffs : Vec<Buff>,// this one is not serializable  normally
    pub buffs_summary : [u8;5], // this one is serialized but not saved 10 bytes

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

        end = start + 4;
        let mob_id_bytes = u32::to_le_bytes(self.mob_id); // 2 bytes
        buffer[start..end].copy_from_slice(&mob_id_bytes);
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
        let origin_tile_id = self.start_position_id.to_bytes(); // 6 bytes
        buffer[start..end].copy_from_slice(&origin_tile_id);
        start = end;

        // end = start + 6;
        // let end_tile_id = self.end_position_id.to_bytes(); // 6 bytes
        // buffer[start..end].copy_from_slice(&end_tile_id);
        // start = end;

        for path_point in self.path
        {
            end = start + 1;
            buffer[start] = path_point;
            start = end;
        }

        // end = start + 6;
        // let target_id_bytes = self.target_id.to_bytes(); // 6 bytes
        // buffer[start..end].copy_from_slice(&target_id_bytes);
        // start = end;

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

impl MobEntity
{
    pub fn init_stats(&mut self, definitions: &Definitions)
    {
        let profession_id = definitions.mobs
            .get(self.mob_definition_id as usize)
            .map_or(0, |m| m.profession);

        let profession = definitions.professions_by_id.get(profession_id as usize);
        let (str_init, vit_init, agi_init, will_init) = profession
            .map_or((0, 0, 0, 0), |p| (p.strength.init, p.vitality.init, p.agility.init, p.will.init));

        let entry = definitions.mob_progression_by_mob
            .get(self.mob_definition_id as usize)
            .and_then(|p| p.get(self.level as usize));

            // asumimos lo maximo que podria ganar si fuera un jugador, luego lo reducimos y le aplicamos una reduccion
        let max_possible_stat_increase = self.level * 4;
        let mut random_generator = <StdRng as rand::SeedableRng>::from_entropy();
        let reduction_effect = rand::Rng::gen::<f32>(&mut random_generator) * 2f32 - 1f32;
        let stat_increase = max_possible_stat_increase as u16 + (max_possible_stat_increase as f32 * 0.5f32 * reduction_effect).floor() as u16;

        self.strength_stat = str_init + stat_increase;
        self.vitality_stat = vit_init + stat_increase;
        self.agility_stat  = agi_init + stat_increase;
        self.will_stat     = will_init + stat_increase;

        self.health = self.get_max_hp(definitions);
    }
}

impl AbilityUser for MobEntity
{
    fn get_health(&self) -> u16
    {
        self.health
    }

    fn update_health(&mut self, new_health: u16, definition: &Definitions)
    {
        self.health = new_health.min(self.get_max_hp(definition));
        cli_log::info!("---- updated mob health {}", self.health)
    }

    fn get_stats(&self) -> (u16, u16, u16, u16)
    {
        (self.strength_stat, self.vitality_stat, self.agility_stat, self.will_stat)
    }

    fn get_mana(&self) -> u16 { 0 }

    fn get_max_mana(&self, _definition: &Definitions) -> u16 { 0 }

    fn get_intelligence(&self) -> u16 { 0 }

    fn get_max_intelligence(&self, _definition: &Definitions) -> u16 { 0 }

    fn get_stamina(&self) -> u16 { 0 }

    fn get_max_stamina(&self, _definition: &Definitions) -> u16 { 0 }

    fn get_hp(&self) -> u16 { self.health }

    fn get_max_hp(&self, definition: &Definitions) -> u16
    {
        let realm = definition
            .mob_progression_by_mob
            .get(self.mob_definition_id as usize)
            .and_then(|p| p.get(self.level as usize))
            .map_or(0, |p| p.realm);

        let multiplier = definition.realms.get(realm as usize)
            .map_or(1f32, |d| d.multiplier);

        let result = self.vitality_stat as f32 * multiplier;
        result.round() as u16
    }
}