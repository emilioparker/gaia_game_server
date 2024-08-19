use crate::map::tetrahedron_id::TetrahedronId;


#[derive(Debug, Clone)]
pub enum MobCommandInfo 
{
    Touch(),
    Spawn(u16, u32, u8), // character id, definition id
    ControlMapEntity(u16),
    Attack(u16, u32, u32, u8, u8), // character id, card id, time, active_effect, missed
    AttackWalker(u16, u32, u32, u8, u8), // character id, card id, time, active_effect
}

#[derive(Debug, Clone)]
pub struct MobCommand 
{
    pub tile_id : TetrahedronId,
    pub info : MobCommandInfo
}