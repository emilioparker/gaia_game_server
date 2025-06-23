use bson::oid::ObjectId;

use crate::map::tetrahedron_id::TetrahedronId;

pub const KINGDOM_ENTITY_SIZE: usize = 9;

#[derive(Debug)]
#[derive(Clone)]
pub struct KingdomEntity 
{
    pub object_id: Option<ObjectId>,
    pub version: u16, // 2 bytes
    pub tetrahedron_id : TetrahedronId, // 6 bytes
    pub faction:u8, // 1
}

impl KingdomEntity 
{
    pub fn to_bytes(&self) -> [u8;KINGDOM_ENTITY_SIZE] 
    {
        let mut buffer = [0u8; KINGDOM_ENTITY_SIZE];
        let mut offset = 0;
        let mut end;

        end = offset + 2;
        let version_bytes = u16::to_le_bytes(self.version); // 2 bytes
        buffer[..end].copy_from_slice(&version_bytes);
        offset = end;

        end = offset + 6;
        let tile_id = self.tetrahedron_id.to_bytes(); // 6 bytes
        buffer[offset..end].copy_from_slice(&tile_id);
        offset = end;

        end = offset + 1;
        buffer[offset] = self.faction;
        offset = end;

        buffer
    }
}