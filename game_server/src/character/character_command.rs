use crate::map::tetrahedron_id::TetrahedronId;

pub const IDLE_ACTION: u8 = 0;
pub const WALK_ACTION: u8 = 1;
pub const ATTACK_TILE_ACTION: u8 = 2;
pub const ATTACK_ACTION: u8 = 3;
pub const COLLECT_ACTION: u8 = 4;
pub const BUILD_ACTION: u8 = 5;
pub const TOUCH: u8 = 6;

//    let info = MapCommandInfo::AttackMob(player_id, card_id, required_time, active_effect);
#[derive(Debug, Clone)]
pub enum CharacterCommandInfo 
{
    Touch(),
    Disconnect(),
    Movement(CharacterMovement),
    Action(u8),
    AttackCharacter(u16, u32, u32, u8), // other_character_id, card_id, required_time, effect
    Greet(),
    Respawn(TetrahedronId),
    SellItem(u8,u32, u16),
    BuyItem(u8,u32, u16),
    UseItem(u8,u32, u16),
    EquipItem(EquipItemCommandData),
    ActivateBuff(u32),
}

#[derive(Debug, Clone)]
pub struct CharacterCommand 
{
    pub player_id : u16,
    pub info : CharacterCommandInfo
}

#[derive(Debug, Clone)]
pub struct CharacterMovement 
{
    pub player_id: u16,
    pub position: TetrahedronId,
    pub second_position: TetrahedronId,
    pub vertex_id: i32,
    pub path: [u8;6],
    pub time: u32,
}

#[derive(Debug, Clone)]
pub struct EquipItemCommandData 
{
    pub faction: u8,
    pub item_id:u32,
    pub current_slot:u8,
    pub new_slot:u8,
}
