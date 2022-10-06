use serde_json::map;

use super::tetrahedron_id::TetrahedronId;

#[derive(Debug, Clone)]
pub struct MapEntity {
    pub id : TetrahedronId,
    pub last_update: u32,
    pub health:u32,
    pub prop: u32,
}

#[derive(Debug, Clone)]
pub enum MapCommandInfo {
    Touch(),
    ChangeHealth(u16),
}

#[derive(Debug, Clone)]
pub struct MapCommand {
    pub id : TetrahedronId,
    pub info : MapCommandInfo
}

impl MapEntity {
    pub fn to_bytes(&self) -> [u8;18] {
        let mut buffer = [0u8; 18];
        let mut start : usize;
        let mut end : usize;

        start = 0;
        end = start + 6;
        let tile_id = self.id.to_bytes(); // 6 bytes
        buffer[start..end].copy_from_slice(&tile_id);

        start = end;
        end = start +4;
        let last_update = u32::to_le_bytes(self.last_update); // 4 bytes
        buffer[start..end].copy_from_slice(&last_update);

        start = end;
        end = start +4;
        let health = u32::to_le_bytes(self.health); // 4 bytes
        buffer[start..end].copy_from_slice(&health);

        start = end;
        end = start +4;
        let prop = u32::to_le_bytes(self.prop); // 4 bytes
        buffer[start..end].copy_from_slice(&prop);

        buffer
    }
}


impl MapCommand {
    pub fn from_bytes(data: &[u8;508]) -> Self {
        let mut start : usize = 0;
        let mut end : usize = 8;

        start = 1; // ignoring first byte
        end = start + 6;

        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let tile_id = TetrahedronId::from_bytes(&buffer);

        start = end;
        end = start + 2;

        let damage = u16::from_le_bytes(data[start..end].try_into().unwrap()); // 2 bytes

        let info = MapCommandInfo::ChangeHealth(damage);
        MapCommand { id: tile_id, info }
    }
}