

#[derive(Debug)]
#[derive(Clone)]
pub struct PlayerState {
    pub current_time: u64,
    pub sequence_number: u64,
    pub player_id: u64,
    pub position: [f32;3],
    pub second_position: [f32;3],
    pub action:u32
}

impl PlayerState {
    pub fn to_bytes(&self) -> [u8;36] {
        let mut buffer = [0u8; 36];

        let player_id_bytes = u64::to_le_bytes(self.player_id); // 8 bytes
        buffer[..8].copy_from_slice(&player_id_bytes);

        float_into_buffer(&mut buffer, self.position[0], 8, 12);
        float_into_buffer(&mut buffer, self.position[1], 12, 16);
        float_into_buffer(&mut buffer, self.position[2], 16, 20);

        float_into_buffer(&mut buffer, self.second_position[0], 20, 24);
        float_into_buffer(&mut buffer, self.second_position[1], 24, 28);
        float_into_buffer(&mut buffer, self.second_position[2], 28, 32);
        
        let action_bytes = u32::to_le_bytes(self.action); // 4 bytes
        buffer[32..36].copy_from_slice(&action_bytes);

        buffer
    }
}

fn float_into_buffer(buffer : &mut [u8;36], data: f32, start : usize, end: usize)
{
    let bytes = f32::to_le_bytes(data);
    buffer[start..end].copy_from_slice(&bytes);
}