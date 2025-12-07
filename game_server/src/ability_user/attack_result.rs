use crate::map::tetrahedron_id::TetrahedronId;

pub const ATTACK_RESULT_SIZE: usize = 26;

pub const NORMAL_ATTACK_RESULT: u8 = 0;
pub const BLOCKED_ATTACK_RESULT: u8 = 1;
pub const MISSED_ATTACK_RESULT: u8 = 2;
pub const CRITICAL_ATTACK_RESULT: u8 = 3;

pub const BATTLE_MOB_CHAR: u8 = 0;
pub const BATTLE_CHAR_MOB: u8 = 1;
pub const BATTLE_CHAR_CHAR: u8 = 2;
pub const BATTLE_MOB_MOB: u8 = 3;
pub const BATTLE_CHAR_TOWER: u8 = 4;


#[derive(Debug, Clone)]
pub struct AttackResult
{
    pub id:u16,// 2 bytes
    pub card_id:u32,// 4 bytes
    pub attacker_character_id: u16, // 2 bytes
    pub attacker_mob_id: u32, // 4 bytes // sometimes we will throw arrows to mobs or even trees I guess.
    pub target_character_id: u16, // 2 bytes
    pub target_mob_id: u32, // 4 bytes // sometimes we will throw arrows to mobs or even trees I guess.
    pub target_tile_id: TetrahedronId, // 6 bytes // sometimes we will throw arrows to mobs or even trees I guess.
    pub battle_type: u8, // 1 byte
    pub result: u8, //1 byte
}

impl AttackResult 
{
    // used by the test_client ignores the protocol byte.
    pub fn to_bytes(&self) -> [u8;ATTACK_RESULT_SIZE] 
    {
        let mut buffer = [0u8; ATTACK_RESULT_SIZE];

        let mut start : usize = 0;
        let mut end : usize = 2;

        let id_bytes = u16::to_le_bytes(self.id); // 2 bytes
        buffer[start..end].copy_from_slice(&id_bytes);
        start = end;

        end = start + 4;
        let card_id_bytes = u32::to_le_bytes(self.card_id); // 4 bytes
        buffer[start..end].copy_from_slice(&card_id_bytes);
        start = end;

        end = start + 2;
        let attacker_character_id_bytes = u16::to_le_bytes(self.attacker_character_id); // 2 bytes
        buffer[start..end].copy_from_slice(&attacker_character_id_bytes);
        start = end;

        end = start + 4;
        let attacker_mob_id_bytes = u32::to_le_bytes(self.attacker_mob_id); // 4 bytes
        buffer[start..end].copy_from_slice(&attacker_mob_id_bytes);
        start = end;

        end = start + 2;
        let target_character_id_bytes = u16::to_le_bytes(self.target_character_id); // 2 bytes
        buffer[start..end].copy_from_slice(&target_character_id_bytes);
        start = end;

        end = start + 4;
        let target_mob_id_bytes = u32::to_le_bytes(self.target_mob_id); // 4 bytes
        buffer[start..end].copy_from_slice(&target_mob_id_bytes);
        start = end;

        end = start + 6;
        let target_tile_id_bytes = self.target_tile_id.to_bytes();
        buffer[start..end].copy_from_slice(&target_tile_id_bytes);
        start = end;

        end = start + 1;
        buffer[start] = self.battle_type;
        start = end;

        end = start + 1;
        buffer[start] = self.result;
        // start = end;

        buffer
    }

    pub fn get_size() -> usize 
    {
        ATTACK_RESULT_SIZE
    }
}