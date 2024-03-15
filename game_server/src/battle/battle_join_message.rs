use crate::map::tetrahedron_id::TetrahedronId;

pub const BATTLE_JOIN_MESSAGE_SIZE: usize = 10;

#[derive(Debug, Clone)]
pub struct BattleJoinMessage
{
    pub target_tile_id: TetrahedronId, // 6 bytes
    pub player_id: u16, // 4 bytes
    pub participation_id: u8, // 1 bytes
    pub result : u8, // 1 bytes
}

impl BattleJoinMessage
{
    pub fn to_bytes(&self) -> [u8;BATTLE_JOIN_MESSAGE_SIZE] 
    {
        let mut buffer = [0u8;BATTLE_JOIN_MESSAGE_SIZE];
        let mut start : usize;
        let mut end : usize;

        start = 0;
        end = start + 6;
        let tile_id = self.target_tile_id.to_bytes(); // 6 bytes
        buffer[start..end].copy_from_slice(&tile_id);

        start = end;
        end = start + 2; 
        let player_bytes = u16::to_le_bytes(self.player_id); // 4 bytes
        buffer[start..end].copy_from_slice(&player_bytes);

        start = end;
        end = start + 1;
        buffer[start] = self.participation_id;

        start = end;
        // end = start + 1;
        buffer[start] = self.result;

        buffer
    }
}