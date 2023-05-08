pub const PLAYER_ATTACK_SIZE: usize = 24;

#[derive(Debug, Clone)]
pub struct PlayerAttack {
    pub player_id: u64, // 8 bytes
    pub target_player_id: u64, // 8 bytes
    pub damage: u32, // 4 bytes
    pub skill_id: u32 // 4 bytes
}

impl PlayerAttack {
    // used by the test_client ignores the protocol byte.
    pub fn to_bytes(&self) -> [u8;24] {
        let mut buffer = [0u8; 24];

        let mut start : usize = 0;
        let mut end : usize = 8;

        let player_id_bytes = u64::to_le_bytes(self.player_id); // 8 bytes
        buffer[start..end].copy_from_slice(&player_id_bytes);
        start = end;
        end = start + 8;

        let target_player_id_bytes = u64::to_le_bytes(self.target_player_id); // 8 bytes
        buffer[start..end].copy_from_slice(&target_player_id_bytes);
        start = end;

        end = start + 4;
        u32_into_buffer(&mut buffer,self.damage, &mut start, end);
        end = start + 4;
        u32_into_buffer(&mut buffer,self.skill_id, &mut start, end);
        buffer
    }

    pub fn from_bytes(data: &[u8;508]) -> Self {

        //1 - protocolo 1 bytes
        //2 - id 8 bytes
        // the rest depends on the code.

        // we are ignoring the first byte because of the protocol
        let mut start = 1;
        let mut end = start + 8;

        let player_id = u64::from_le_bytes(data[start..end].try_into().unwrap());
        start = end;

        end = start + 8;
        let target_player_id = u64::from_le_bytes(data[start..end].try_into().unwrap());
        start = end;

        // 1 byte + 8 bytes + 1 byte + 4x3:12 bytes + 4x3:12 bytes + 4 bytes = 18 bytes
        end = start + 4;
        let damage = decode_u32(data, &mut start, end);
        end = start + 4;
        let skill_id = decode_u32(data, &mut start, end);

        PlayerAttack { player_id, target_player_id, damage, skill_id}
    }
}

pub fn decode_u32(buffer: &[u8;508], start: &mut usize, end: usize) -> u32
{
    let decoded_u32 = u32::from_le_bytes(buffer[*start..(*start + 4)].try_into().unwrap());
    *start = end;
    decoded_u32
}

fn u32_into_buffer(buffer : &mut [u8;24], data: u32, start : &mut usize, end: usize)
{
    let bytes = u32::to_le_bytes(data);
    buffer[*start..end].copy_from_slice(&bytes);
    *start = end;
}