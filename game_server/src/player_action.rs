use crate::tetrahedron_id::TetrahedronId;


pub const PLAYER_ACTIVITY_CODE: u8 = 1;
pub const PLAYER_INTERACTION_CODE: u8 = 2; 

#[derive(Debug)]
pub enum PlayerAction {
    Activity(PlayerActivity),
    Interaction(PlayerEntityInteraction),
}

#[derive(Debug)]
pub struct PlayerEntityInteraction {
    pub player_id: u64,
    pub tile_id: TetrahedronId,
    pub damage:u16,
}

#[derive(Debug)]
pub struct PlayerActivity {
    pub player_id: u64,
    pub position: [f32;3],
    pub direction: [f32;3],
    pub action:u32,
}

impl PlayerAction {
    // used by the test_client ignores the protocol byte.
    pub fn to_bytes(&self) -> [u8;37] {
        let mut buffer = [0u8; 37];

        match self {
            PlayerAction::Activity(data) => {
                let mut start : usize = 0;
                let mut end : usize = 8;

                let player_id_bytes = u64::to_le_bytes(data.player_id); // 8 bytes
                buffer[start..end].copy_from_slice(&player_id_bytes);
                start = end;
                end = start + 1;

                buffer[start] = PLAYER_ACTIVITY_CODE;
                start = end;

                float_into_buffer(&mut buffer, data.position[0], &mut start, end);
                end = start + 4;
                float_into_buffer(&mut buffer, data.position[1], &mut start, end);
                end = start + 4;
                float_into_buffer(&mut buffer, data.position[2], &mut start, end);
                end = start + 4;

                float_into_buffer(&mut buffer, data.direction[0], &mut start, end);
                end = start + 4;
                float_into_buffer(&mut buffer, data.direction[1], &mut start, end);
                end = start + 4;
                float_into_buffer(&mut buffer, data.direction[2], &mut start, end);

                end = start + 4;
                let action_bytes = u32::to_le_bytes(data.action); // 4 bytes
                buffer[start..end].copy_from_slice(&action_bytes);

                buffer
            },
            PlayerAction::Interaction(data) => {

                let mut start : usize = 0;
                let mut end : usize = 8;

                let player_id_bytes = u64::to_le_bytes(data.player_id); // 8 bytes
                buffer[start..end].copy_from_slice(&player_id_bytes);
                start = end;
                end = start + 1;

                buffer[start] = PLAYER_INTERACTION_CODE;
                start = end;

                end = start + 6;
                let bytes = data.tile_id.to_bytes();
                buffer[start..end].copy_from_slice(&bytes);

                start = end;

                end = start + 2;
                let damage_bytes = u16::to_le_bytes(data.damage); // 2 bytes
                buffer[start..end].copy_from_slice(&damage_bytes);

                buffer
            },
        }

    }

    pub fn from_bytes(data: &[u8;508]) -> Self {

        //1 - protocolo 1 bytes
        //2 - id 8 bytes
        //3 - code 1 byte
        // the rest depends on the code.

        // we are ignoring the first byte because of the protocol
        let mut start = 1;
        let mut end = start + 8;

        let player_id = u64::from_le_bytes(data[start..end].try_into().unwrap());
        start = end;
        end = start + 1;

        let action_code = data[start];
        start = end;

        if action_code == PLAYER_ACTIVITY_CODE
        {
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

            end = start + 4;
            let action = u32::from_le_bytes(data[start..end].try_into().unwrap());

            let client_action = PlayerActivity {
                player_id,
                position,
                direction,
                action
            };

            PlayerAction::Activity(client_action)
        }
        else {
            // 1 byte + 8 bytes + 1 byte + 6 bytes + 2 bytes = 18 bytes
            end = start + 6;
            let mut buffer = [0u8;6];
            buffer.copy_from_slice(&data[start..end]);
            let tile_id = TetrahedronId::from_bytes(&buffer);

            start = end;
            end = start + 2;

            let damage = u16::from_le_bytes(data[start..end].try_into().unwrap()); // 2 bytes

            let client_action = PlayerEntityInteraction {
                player_id,
                tile_id,
                damage
            };

            PlayerAction::Interaction(client_action)
            
        }
    }
}

pub fn decode_float(buffer: &[u8;508], start: &mut usize, end: usize) -> f32
{
    let decoded_float = f32::from_le_bytes(buffer[*start..(*start + 4)].try_into().unwrap());
    *start = end;

    decoded_float
}

fn float_into_buffer(buffer : &mut [u8;37], data: f32, start : &mut usize, end: usize)
{
    let bytes = f32::to_le_bytes(data);
    buffer[*start..end].copy_from_slice(&bytes);
    *start = end;
}