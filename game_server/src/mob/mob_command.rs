use crate::map::tetrahedron_id::{self, TetrahedronId};


#[derive(Debug, Clone)]
pub enum MobCommandInfo 
{
    Touch(),
    Spawn(u16, u32, u8), // character id, definition id
    ControlMob(u16),
    MoveMob(u16, TetrahedronId, TetrahedronId, [u8;6]),
    CastFromCharacterToMob(u16, u32, u32, u8, u8), // character id, card id, time, active_effect, missed
    CastFromMobToMob(TetrahedronId, u32, u32, u8, u8), // caster_mob_id, card id, time, active_effect, missed
    AttackFromMobToWalker(u16, u32, u32, u8, u8), // character id, card id, time, active_effect
}

#[derive(Debug, Clone)]
pub struct MobCommand 
{
    pub tile_id : TetrahedronId,
    pub info : MobCommandInfo
}