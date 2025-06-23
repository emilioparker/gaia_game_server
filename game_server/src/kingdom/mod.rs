pub mod kingdom_entity;

use crate::map::tetrahedron_id::TetrahedronId;

#[derive(Debug, Clone)]
pub enum KingdomCommandInfo 
{
    Touch(),
}

#[derive(Debug, Clone)]
pub struct KingdomCommand 
{
    pub id : TetrahedronId,
    pub info : KingdomCommandInfo
}