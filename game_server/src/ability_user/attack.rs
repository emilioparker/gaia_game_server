use crate::map::tetrahedron_id::TetrahedronId;

pub const ATTACK_SIZE: usize = 21;

#[derive(Debug, Clone)]
pub struct Attack 
{
    pub id:u16,// 2 bytes
    pub character_id: u16, // 2 bytes
    pub target_character_id: u16, // 2 bytes
    pub target_mob_tile_id: TetrahedronId, // 6 bytes // sometimes we will throw arrows to mobs or even trees I guess.
    pub card_id: u32, // 4 bytes
    pub required_time:u32, // 4 bytes
    pub active_effect:u8, //1 byte
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
        let player_id_bytes = u16::to_le_bytes(self.character_id); // 2 bytes
        buffer[start..end].copy_from_slice(&player_id_bytes);
        start = end;

        end = start + 2;
        let target_character_id_bytes = u16::to_le_bytes(self.target_character_id); // 2 bytes
        buffer[start..end].copy_from_slice(&target_character_id_bytes);
        start = end;

        end = start + 6;
        let tile_id_bytes = self.target_mob_tile_id.to_bytes();
        buffer[start..end].copy_from_slice(&tile_id_bytes);
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
        buffer[start] = self.active_effect;
        start = end;

        buffer
    }
}