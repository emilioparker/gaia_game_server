use super::tetrahedron_id::TetrahedronId;

#[derive(Debug, Clone)]
pub struct MapEntity {
    pub id : TetrahedronId,
    pub last_update: u32,
    pub health:u32,
    pub prop: u32,
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