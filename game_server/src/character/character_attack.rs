use crate::map::tetrahedron_id::TetrahedronId;

pub const CHARACTER_ATTACK_SIZE: usize = 14;

#[derive(Debug, Clone)]
pub struct CharacterAttack 
{
    pub player_id: u16, // 2 bytes
    pub target_player_id: u16, // 2 bytes
    pub target_tile_id: TetrahedronId, // 6 bytes // sometimes we will throw arrows to mobs or even trees I guess.
    pub card_id: u32 // 4 bytes
}

impl CharacterAttack 
{
    // used by the test_client ignores the protocol byte.
    pub fn to_bytes(&self) -> [u8;CHARACTER_ATTACK_SIZE] {
        let mut buffer = [0u8; CHARACTER_ATTACK_SIZE];

        let mut start : usize = 0;
        let mut end : usize = 2;

        let player_id_bytes = u16::to_le_bytes(self.player_id); // 2 bytes
        buffer[start..end].copy_from_slice(&player_id_bytes);
        start = end;
        end = start + 2;

        let target_player_id_bytes = u16::to_le_bytes(self.target_player_id); // 2 bytes
        buffer[start..end].copy_from_slice(&target_player_id_bytes);
        start = end;

        end = start + 6;
        let tile_id_bytes = self.target_tile_id.to_bytes();
        buffer[start..end].copy_from_slice(&tile_id_bytes);
        start = end;

        end = start + 4;
        u32_into_buffer(&mut buffer,self.card_id, &mut start, end);
        buffer
    }

    // pub fn from_bytes(data: &[u8;508]) -> Self {

    //     //1 - protocolo 1 bytes
    //     //2 - id 8 bytes
    //     // the rest depends on the code.

    //     // we are ignoring the first byte because of the protocol
    //     let mut start = 1;
    //     let mut end = start + 2;

    //     let player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());
    //     start = end;

    //     end = start + 2;
    //     let target_player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());
    //     start = end;

    //     end = start + 6;
    //     let mut buffer = [0u8;6];
    //     buffer.copy_from_slice(&data[start..end]);
    //     let target_tile_id = TetrahedronId::from_bytes(&buffer);
    //     start = end;

    //     end = start + 4;
    //     let damage = decode_u32(data, &mut start, end);
    //     end = start + 4;
    //     let skill_id = decode_u32(data, &mut start, end);

    //     CharacterAttack { player_id, target_player_id, damage, skill_id, target_tile_id}
    // }
}

pub fn decode_u32(buffer: &[u8;508], start: &mut usize, end: usize) -> u32
{
    let decoded_u32 = u32::from_le_bytes(buffer[*start..(*start + 4)].try_into().unwrap());
    *start = end;
    decoded_u32
}

fn u32_into_buffer(buffer : &mut [u8], data: u32, start : &mut usize, end: usize)
{
    let bytes = u32::to_le_bytes(data);
    buffer[*start..end].copy_from_slice(&bytes);
    *start = end;
}