use crate::map::tetrahedron_id::TetrahedronId;


pub const ATTACK_SIZE: usize = 29;

#[derive(Debug, Clone)]
pub struct Attack 
{
    pub id:u16,// 2 bytes
    pub attacker_hero_id: u16, // 2 bytes
    pub attacker_mob_id: u32, // 4 bytes 
    pub target_hero_id: u16, // 2 bytes
    pub target_mob_id: u32, // 4 bytes // sometimes we will throw arrows to mobs or even trees I guess. can be towers
    pub target_tile_id: TetrahedronId, // 6 bytes // sometimes we will throw arrows to mobs or even trees I guess. can be towers
    pub card_id: u32, // 4 bytes
    pub required_time:u32, // 4 bytes
    pub battle_type: u8, // 1 byte
}

impl Attack 
{
    // used by the test_client ignores the protocol byte.
    pub fn to_bytes(&self) -> [u8;ATTACK_SIZE] 
    {
        let mut buffer = [0u8; ATTACK_SIZE];

        let mut start : usize = 0;
        let mut end : usize = 2;

        let id_bytes = u16::to_le_bytes(self.id); // 2 bytes
        buffer[start..end].copy_from_slice(&id_bytes);
        start = end;

        end = start + 2;
        let attacker_hero_id_bytes = u16::to_le_bytes(self.attacker_hero_id); // 2 bytes
        buffer[start..end].copy_from_slice(&attacker_hero_id_bytes);
        start = end;

        end = start + 4;
        let attacker_mob_id_bytes = u32::to_le_bytes(self.attacker_mob_id); // 2 bytes
        buffer[start..end].copy_from_slice(&attacker_mob_id_bytes);
        start = end;

        end = start + 2;
        let target_hero_id_bytes = u16::to_le_bytes(self.target_hero_id); // 2 bytes
        buffer[start..end].copy_from_slice(&target_hero_id_bytes);
        start = end;

        end = start + 4;
        let target_mob_id_bytes = u32::to_le_bytes(self.target_mob_id); // 2 bytes
        buffer[start..end].copy_from_slice(&target_mob_id_bytes);
        start = end;

        end = start + 6;
        let target_tile_id_bytes = self.target_tile_id.to_bytes();
        buffer[start..end].copy_from_slice(&target_tile_id_bytes);
        start = end;

        end = start + 4;
        let card_id_bytes = u32::to_le_bytes(self.card_id); // 2 bytes
        buffer[start..end].copy_from_slice(&card_id_bytes);
        start = end;

        end = start + 4;
        let end_time_bytes = u32::to_le_bytes(self.required_time); // 4 bytes
        buffer[start..end].copy_from_slice(&end_time_bytes);
        start = end;

        end = start + 1;
        buffer[start] = self.battle_type;
        start = end;

        buffer
    }

    pub fn get_size() -> usize 
    {
        ATTACK_SIZE
    }
}