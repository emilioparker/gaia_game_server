pub const IDLE_ACTION: u32 = 0;
pub const WALK_ACTION: u32 = 1;
pub const ATTACK_TILE_ACTION: u32 = 2;
pub const ATTACK_ACTION: u32 = 3;
pub const COLLECT_ACTION: u32 = 4;
pub const GREET_ACTION: u32 = 5;
pub const RESPAWN_ACTION: u32 = 6;
pub const BUILD_ACTION: u32 = 7;
pub const TOUCH: u32 = 8;

#[derive(Debug, Clone)]
pub enum CharacterCommandInfo 
{
    Touch(),
    Movement(CharacterMovement),
    SellItem(u8,u32, u16),
    BuyItem(u8,u32, u16),
    UseItem(u8,u32, u16),
    EquipItem(EquipItemCommandData),
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
    pub position: [f32;3],
    pub second_position: [f32;3],
    pub other_player_id:u16,
    pub action:u32,
    pub required_time:u32,
    pub skill_id:u32, // if a attack action happens, we need to map that to a skill and calculate the damage.
}

#[derive(Debug, Clone)]
pub struct EquipItemCommandData 
{
    pub faction: u8,
    pub item_id:u32,
    pub current_slot:u8,
    pub new_slot:u8,
}
