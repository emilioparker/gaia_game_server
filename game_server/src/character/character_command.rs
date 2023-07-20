pub const IDLE_ACTION: u32 = 0;
pub const WALK_ACTION: u32 = 1;
pub const ATTACK_TILE_ACTION: u32 = 2;
pub const ATTACK_ACTION: u32 = 3;
pub const COLLECT_ACTION: u32 = 4;
pub const GREET_ACTION: u32 = 5;
pub const RESPAWN_ACTION: u32 = 6;
pub const BUILD_ACTION: u32 = 7;

#[derive(Debug)]
pub struct CharacterCommand {
    pub player_id: u16,
    pub position: [f32;3],
    pub second_position: [f32;3],
    pub other_player_id:u16,
    pub action:u32,
    pub required_time:u32,
    pub skill_id:u32, // if a attack action happens, we need to map that to a skill and calculate the damage.
}

impl CharacterCommand {
    // used by the test_client ignores the protocol byte.
    pub fn to_bytes(&self) -> [u8;34] {
        let mut buffer = [0u8; 34];

        let mut start : usize = 0;
        let mut end : usize = 2;

        let player_id_bytes = u16::to_le_bytes(self.player_id); // 2 bytes
        buffer[start..end].copy_from_slice(&player_id_bytes);
        start = end;
        end = start + 4;

        float_into_buffer(&mut buffer, self.position[0], &mut start, end);
        end = start + 4;
        float_into_buffer(&mut buffer, self.position[1], &mut start, end);
        end = start + 4;
        float_into_buffer(&mut buffer, self.position[2], &mut start, end);
        end = start + 4;

        float_into_buffer(&mut buffer, self.second_position[0], &mut start, end);
        end = start + 4;
        float_into_buffer(&mut buffer, self.second_position[1], &mut start, end);
        end = start + 4;
        float_into_buffer(&mut buffer, self.second_position[2], &mut start, end);

        end = start + 2;
        let other_player_id_bytes = u16::to_le_bytes(self.other_player_id); // 2 bytes
        buffer[start..end].copy_from_slice(&other_player_id_bytes);
        start = end;

        end = start + 4;
        let action_bytes = u32::to_le_bytes(self.action); // 4 bytes
        buffer[start..end].copy_from_slice(&action_bytes);

        buffer
    }

    pub fn from_bytes(data: &[u8;508]) -> Self {

        //1 - protocolo 1 bytes
        //2 - id 8 bytes
        // the rest depends on the code.

        // we are ignoring the first byte because of the protocol
        let mut start = 1;
        let mut end = start + 2;

        let player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());
        start = end;

        end = start + 8;
        let session_id = u64::from_le_bytes(data[start..end].try_into().unwrap());
        start = end;

        // 1 byte + 8 bytes + 1 byte + 4x3:12 bytes + 4x3:12 bytes + 4 bytes = 18 bytes
        end = start + 4;
        let pos_x = decode_float(data, &mut start, end);
        end = start + 4;
        let pos_y = decode_float(data, &mut start, end);
        end = start + 4;
        let pos_z = decode_float(data, &mut start, end);
        end = start + 4;
        let position = [pos_x, pos_y, pos_z];

        let direction_x = decode_float(data, &mut start, end);
        end = start + 4;
        let direction_y = decode_float(data, &mut start, end);
        end = start + 4;
        let direction_z = decode_float(data, &mut start, end);
        let direction = [direction_x, direction_y, direction_z];

        end = start + 2;
        let other_player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());
        start = end;

        end = start + 4;
        let action = u32::from_le_bytes(data[start..end].try_into().unwrap());
        start = end;

        end = start + 4;
        let required_time = u32::from_le_bytes(data[start..end].try_into().unwrap());
        start = end;

        let client_action = CharacterCommand {
            player_id,
            position,
            second_position: direction,
            other_player_id,
            action,
            required_time,
            skill_id: 0,
        };

        client_action
    }
}

pub fn decode_float(buffer: &[u8;508], start: &mut usize, end: usize) -> f32
{
    let decoded_float = f32::from_le_bytes(buffer[*start..(*start + 4)].try_into().unwrap());
    *start = end;

    decoded_float
}

fn float_into_buffer(buffer : &mut [u8], data: f32, start : &mut usize, end: usize)
{
    let bytes = f32::to_le_bytes(data);
    buffer[*start..end].copy_from_slice(&bytes);
    *start = end;
}