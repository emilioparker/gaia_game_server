
use crate::map::tetrahedron_id::TetrahedronId;

pub const CHAT_ENTRY_SIZE: usize = 414;

#[derive(Debug)]
#[derive(Clone)]
pub struct ChatEntry 
{
    pub tetrahedron_id : TetrahedronId, // 6 bytes
    pub timestamp : u32,
    pub faction: u8,
    pub player_id: u16, // 2 bytes
    pub message_length:u8, // 1 bytes
    pub message: [u32;100], //400 bytes
}

impl ChatEntry 
{
    pub fn to_bytes(&self) -> [u8;CHAT_ENTRY_SIZE] 
    {
        let mut buffer = [0u8; CHAT_ENTRY_SIZE];
        let mut offset = 0;
        let mut end;

        end = offset + 6;
        let tile_id = self.tetrahedron_id.to_bytes(); // 6 bytes
        buffer[offset..end].copy_from_slice(&tile_id);
        offset = end;

        end = offset + 4;
        let timestamp_bytes = u32::to_le_bytes(self.timestamp);
        buffer[offset..end].copy_from_slice(&timestamp_bytes);
        offset = end;

        cli_log::info!("enconding chat entry player id {}", self.player_id);
        end = offset + 2;
        let player_id_bytes = u16::to_le_bytes(self.player_id); // 2 bytes
        buffer[offset..end].copy_from_slice(&player_id_bytes);
        offset = end;

        end = offset + 1;
        buffer[offset] = self.faction;
        offset = end;

        end = offset + 1;
        buffer[offset] = self.message_length;
        offset = end;

        for i in 0..(self.message_length as usize)
        {
            end = offset + 4;
            u32_into_buffer(&mut buffer,self.message[i], &mut offset, end);
        }

        buffer
    }

}

fn u32_into_buffer(buffer : &mut [u8], data: u32, start : &mut usize, end: usize)
{
    let bytes = u32::to_le_bytes(data);
    buffer[*start..end].copy_from_slice(&bytes);
    *start = end;
}