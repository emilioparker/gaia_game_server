use crate::map::tetrahedron_id::TetrahedronId;

pub mod tower_entity;

#[derive(Debug, Clone)]
pub enum TowerCommandInfo 
{
    Touch(),
    AttackTower(u16,u16,u8,u32,u32),
    RepairTower(u16,u8,u16),
}

#[derive(Debug, Clone)]
pub struct TowerCommand 
{
    pub id : TetrahedronId,
    pub info : TowerCommandInfo
}