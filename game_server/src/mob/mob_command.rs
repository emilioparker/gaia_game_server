use crate::map::tetrahedron_id::TetrahedronId;


#[derive(Debug, Clone)]
pub enum MobCommand
{
    Touch(TouchMobData),
    Spawn(SpawnMobData), // character id, definition id
    ControlMob(ControlMobData),
    MoveMob(MoveMobData),
    CastFromHeroToMob(HeroToMobData), // character id, card id, time, active_effect, missed
    CastFromMobToMob(MobToMobData), // caster_mob_id, card id, time, active_effect, missed
    AttackFromMobToHero(MobToHeroData), // character id, card id, time, active_effect
}

// #[derive(Debug, Clone)]
// pub struct MobCommand 
// {
//     pub tile_id : TetrahedronId,
//     pub info : MobCommandInfo
// }

#[derive(Debug, Clone)]
pub struct MobToMobData 
{
    pub card_id : u32,
    pub time : u32,
    // pub active_effect : u8,
    pub missed : u8,
    pub caster_mob_id : u32,
    pub caster_mob_tile_id : TetrahedronId,
    pub target_mob_id : u32,
    pub target_mob_tile_id : TetrahedronId,
}

#[derive(Debug, Clone)]
pub struct HeroToMobData 
{
    pub hero_id :u16,
    pub card_id : u32,
    pub time : u32,
    // pub active_effect : u8,
    pub missed : u8,
    pub target_mob_id : u32,
    pub target_mob_tile_id : TetrahedronId,
}

    // AttackFromMobToHero(u16, u32, u32, u8, u8), // character id, card id, time, active_effect

#[derive(Debug, Clone)]
pub struct MobToHeroData 
{
    pub hero_id :u16,
    pub card_id : u32,
    pub time : u32,
    // pub active_effect : u8,
    pub missed : u8,
    pub attacker_mob_id : u32,
    pub attacker_mob_tile_id : TetrahedronId,
}

// MoveMob(u16, TetrahedronId, TetrahedronId, [u8;6]),

#[derive(Debug, Clone)]
pub struct MoveMobData 
{
    pub hero_id : u16,
    pub mob_id : u32,
    pub new_origin_tile_id : TetrahedronId,
    pub new_end_tile_id : TetrahedronId,
    pub path : [u8;6]
}

// ControlMob(u16),

#[derive(Debug, Clone)]
pub struct ControlMobData 
{
    pub hero_id : u16,
    pub mob_id : u32,
    pub mob_tile_id : TetrahedronId,
}

    // Spawn(u16, u32, u8), // character id, definition id
#[derive(Debug, Clone)]
pub struct SpawnMobData 
{
    pub hero_id : u16,
    pub mob_definition_id : u32,
    pub tile_id : TetrahedronId,
    pub level : u8
}

#[derive(Debug, Clone)]
pub struct TouchMobData 
{
    pub mob_id : u32,
    pub mob_tile_id : TetrahedronId,
}