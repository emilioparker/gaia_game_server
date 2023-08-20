use crate::map::tetrahedron_id::TetrahedronId;

pub mod tower_entity;

#[derive(Debug, Clone)]
pub enum TowerCommandInfo 
{
    Touch(),
    AttackTower(u16,u8,u16, u32), // faction might be unnecessary
}

#[derive(Debug, Clone)]
pub struct TowerCommand 
{
    pub id : TetrahedronId,
    pub info : TowerCommandInfo
}