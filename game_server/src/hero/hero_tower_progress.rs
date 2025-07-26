use crate::{long_term_storage_service::db_hero::StoredTowerProgress, map::tetrahedron_id::TetrahedronId};

pub const HERO_TOWER_PROGRESS_SIZE: usize = 24;

#[derive(Debug, Clone)]
pub struct HeroTowerProgress
{
    pub id : TetrahedronId, // 6
    pub tower_floor: u32, //4
    pub start_time : u64, // 8
    pub points : u32 // 4
}

impl HeroTowerProgress 
{
    // used by the test_client ignores the protocol byte.
    pub fn to_bytes(&self) -> [u8;24] 
    {
        let mut buffer = [0u8; 24];

        let mut start : usize = 0;
        let mut end : usize = 2;

        end = start + 6;
        let tile_id = self.id.to_bytes(); // 6 bytes
        buffer[start..end].copy_from_slice(&tile_id);
        start = end;

        u32_into_buffer(&mut buffer,self.tower_floor, &mut start, end);
        end = start + 4;

        let start_time_bytes = u64::to_le_bytes(self.start_time); // 8 bytes
        buffer[start..end].copy_from_slice(&start_time_bytes);
        start = end;

        end = start + 4;
        u32_into_buffer(&mut buffer,self.points, &mut start, end);
        buffer
    }


    pub fn default() -> Self
    {
        HeroTowerProgress { id: TetrahedronId ::default(), tower_floor: 0, start_time: 0, points: 0 }
    }

    pub fn get_size() -> usize 
    {
        HERO_TOWER_PROGRESS_SIZE
    }
}

impl From<StoredTowerProgress> for HeroTowerProgress
{
    fn from(stored_data: StoredTowerProgress) -> Self
    {
        HeroTowerProgress 
        { 
            id: TetrahedronId::from_string(&stored_data.id),
            tower_floor: stored_data.tower_floor,
            start_time: stored_data.start_time,
            points: stored_data.points 
        }
    }
}

fn u32_into_buffer(buffer : &mut [u8], data: u32, start : &mut usize, end: usize)
{
    let bytes = u32::to_le_bytes(data);
    buffer[*start..end].copy_from_slice(&bytes);
    *start = end;
}