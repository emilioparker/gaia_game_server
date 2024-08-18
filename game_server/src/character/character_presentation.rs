pub const CHARACTER_PRESENTATION_SIZE: usize = 22;

#[derive(Debug, Clone)]
pub struct CharacterPresentation {
    pub player_id: u16, // 2 bytes
    pub character_name: [u32;5], //20 bytes
}

impl CharacterPresentation {
    // used by the test_client ignores the protocol byte.
    pub fn to_bytes(&self) -> [u8;22] {
        let mut buffer = [0u8; 22];

        let mut start : usize = 0;
        let mut end : usize = 2;

        let player_id_bytes = u16::to_le_bytes(self.player_id); // 2 bytes
        buffer[start..end].copy_from_slice(&player_id_bytes);
        start = end;
        end = start + 4;

        u32_into_buffer(&mut buffer,self.character_name[0], &mut start, end);
        end = start + 4;
        u32_into_buffer(&mut buffer,self.character_name[1], &mut start, end);
        end = start + 4;
        u32_into_buffer(&mut buffer,self.character_name[2], &mut start, end);
        end = start + 4;
        u32_into_buffer(&mut buffer,self.character_name[3], &mut start, end);
        end = start + 4;
        u32_into_buffer(&mut buffer,self.character_name[4], &mut start, end);
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

        // 1 byte + 8 bytes + 1 byte + 4x3:12 bytes + 4x3:12 bytes + 4 bytes = 18 bytes
        end = start + 4;
        let a = decode_u32(data, &mut start, end);
        end = start + 4;
        let b = decode_u32(data, &mut start, end);
        end = start + 4;
        let c = decode_u32(data, &mut start, end);
        end = start + 4;
        let d = decode_u32(data, &mut start, end);
        end = start + 4;
        let e = decode_u32(data, &mut start, end);

        CharacterPresentation { player_id, character_name: [a,b,c,d,e] }
    }

    pub fn get_size() -> usize 
    {
        CHARACTER_PRESENTATION_SIZE
    }
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