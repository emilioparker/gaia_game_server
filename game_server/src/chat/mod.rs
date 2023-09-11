use crate::map::tetrahedron_id::TetrahedronId;

pub mod chat_entry;

// #[derive(Debug, Clone)]
// pub enum TowerCommandInfo 
// {
//     Touch(),
//     AttackTower(u16,u16, u32),
//     RepairTower(u16,u16),
// }

#[derive(Debug, Clone)]
pub struct ChatCommand 
{
    pub id : TetrahedronId,
    pub faction: u8,
    pub player_id: u16, // 2 bytes
    pub message_length: u8, // 1 bytes
    pub message: [u32; 100],
}