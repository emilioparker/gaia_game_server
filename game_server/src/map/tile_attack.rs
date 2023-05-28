use super::tetrahedron_id::TetrahedronId;

pub const TILE_ATTACK_SIZE: usize = 16;

#[derive(Debug, Clone)]
pub struct TileAttack {
    pub tile_id: TetrahedronId, // 6 bytes
    pub target_player_id: u16, // 2 bytes
    pub damage: u32, // 4 bytes
    pub skill_id: u32 // 4 bytes
}

impl TileAttack {
    pub fn to_bytes(&self) -> [u8;16] {
        let mut buffer = [0u8; 16];

        let mut start : usize = 0;
        let mut end : usize = 0;

        start = 0;
        end = start + 6;
        let tile_id = self.tile_id.to_bytes(); // 6 bytes
        buffer[start..end].copy_from_slice(&tile_id);
        start = end;

        end = start + 2;
        let player_id_bytes = u16::to_le_bytes(self.target_player_id); // 2 bytes
        buffer[start..end].copy_from_slice(&player_id_bytes);
        start = end;

        end = start + 4;
        u32_into_buffer(&mut buffer,self.damage, &mut start, end);
        end = start + 4;
        u32_into_buffer(&mut buffer,self.skill_id, &mut start, end);
        buffer
    }
}


fn u32_into_buffer(buffer : &mut [u8], data: u32, start : &mut usize, end: usize)
{
    let bytes = u32::to_le_bytes(data);
    buffer[*start..end].copy_from_slice(&bytes);
    *start = end;
}