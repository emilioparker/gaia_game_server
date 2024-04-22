use crate::map::tetrahedron_id::TetrahedronId;


#[derive(Debug, Clone)]
pub enum BattleCommandInfo 
{
    Touch(),
    Join(),
    Attack(u8, u32),
}

#[derive(Debug, Clone)]
pub struct BattleCommand 
{
    pub tile_id : TetrahedronId,
    pub player_id : u16,
    pub info : BattleCommandInfo
}