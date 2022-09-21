

#[derive(Debug)]
pub struct ClientAction {
    pub player_id: u64,
    pub position: [f32;3],
    pub direction: [f32;3],
    pub action:u32,
}

impl ClientAction {
    pub fn to_bytes(&self) -> [u8;36] {
        let mut buffer = [0u8; 36];

        let player_id_bytes = u64::to_le_bytes(self.player_id); // 8 bytes
        buffer[..8].copy_from_slice(&player_id_bytes);

        float_into_buffer(&mut buffer, self.position[0], 8, 12);
        float_into_buffer(&mut buffer, self.position[1], 12, 16);
        float_into_buffer(&mut buffer, self.position[2], 16, 20);

        float_into_buffer(&mut buffer, self.direction[0], 20, 24);
        float_into_buffer(&mut buffer, self.direction[1], 24, 28);
        float_into_buffer(&mut buffer, self.direction[2], 28, 32);
        
        let action_bytes = u32::to_le_bytes(self.action); // 4 bytes
        buffer[32..36].copy_from_slice(&action_bytes);

        buffer
    }

    pub fn from_bytes(data: &[u8;508]) -> Self {
        let mut start = 1;
        let mut end = start + 8;

        let player_id = u64::from_le_bytes(data[start..end].try_into().unwrap());
        start = end;
        end = start + 4;

        let pos_x = decode_float(data, &mut start, &mut end);
        let pos_y = decode_float(data, &mut start, &mut end);
        let pos_z = decode_float(data, &mut start, &mut end);
        let position = [pos_x, pos_y, pos_z];

        let direction_x = decode_float(data, &mut start, &mut end);
        let direction_y = decode_float(data, &mut start, &mut end);
        let direction_z = decode_float(data, &mut start, &mut end);
        let direction = [direction_x, direction_y, direction_z];

        let action = u32::from_le_bytes(data[start..(start + 4)].try_into().unwrap());

        let client_action = ClientAction {
            player_id,
            position,
            direction,
            action
        };

        client_action
    }
}

pub fn decode_float(buffer: &[u8;508], start: &mut usize, end: &mut usize) -> f32
{
    let decoded_float = f32::from_le_bytes(buffer[*start..(*start + 4)].try_into().unwrap());
    *start = *end;
    *end = *start + 4;

    decoded_float
}

fn float_into_buffer(buffer : &mut [u8;36], data: f32, start : usize, end: usize)
{
    let bytes = f32::to_le_bytes(data);
    buffer[start..end].copy_from_slice(&bytes);
}