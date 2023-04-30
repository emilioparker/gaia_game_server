use bson::oid::ObjectId;

use super::tetrahedron_id::TetrahedronId;

pub const MAP_ENTITY_SIZE: usize = 56;

#[derive(Debug, Clone, PartialEq)]
pub struct MapEntity { // 56 bytes
    pub object_id : Option<ObjectId>,
    pub id : TetrahedronId, // 6 bytes
    pub last_update: u32, // 4 bytes
    pub prop: u32, // 4 bytes
    pub faction:u8, // 1 bytes
    pub level:u8,// 1 bytes
    pub temperature:f32, //4 bytes
    pub moisture:f32, //4 bytes
    pub heights : [f32;3], // 12 bytes
    pub pathness : [f32;3], // 12 bytes
    pub health:u32, // 4 bytes
    pub constitution:u32, // 4 bytes
}

impl MapEntity {
    pub fn new(id : &str, health : u32) -> MapEntity {
        
        let entity = MapEntity{
            object_id: None,
            id: TetrahedronId::from_string(id),
            last_update: 1000,
            prop: 10,
            faction: 0,
            level: 0,
            temperature: 1.2,
            moisture: 0.2,
            heights: [0.2,1.0,2.2],
            pathness: [0.0,0.0,0.0],
            health,
            constitution: 100,
        };
          
        entity
    }
    pub fn get_size() -> usize {
        MAP_ENTITY_SIZE
    }
}

#[derive(Debug, Clone)]
pub enum MapCommandInfo {
    Touch(),
    ChangeHealth(u64,u16),
    LayFoundation(u64,u32, f32, f32, f32),
    BuildStructure(u64,u32)
}

#[derive(Debug, Clone)]
pub struct MapCommand {
    pub id : TetrahedronId,
    pub info : MapCommandInfo
}

impl MapEntity {
    pub fn to_bytes(&self) -> [u8;MAP_ENTITY_SIZE] {
        let mut buffer = [0u8;MAP_ENTITY_SIZE];
        let mut start : usize;
        let mut end : usize;

        start = 0;
        end = start + 6;
        let tile_id = self.id.to_bytes(); // 6 bytes
        buffer[start..end].copy_from_slice(&tile_id);
        start = end;

        u32_into_buffer(&mut buffer, self.last_update, &mut start, &mut end);
        u32_into_buffer(&mut buffer, self.prop, &mut start, &mut end);

        buffer[start] = self.faction;
        start += 1;
        end += 1;
        buffer[start] = self.level;
        start += 1;
        end += 1;

        float_into_buffer(&mut buffer, self.temperature, &mut start, &mut end);
        float_into_buffer(&mut buffer, self.moisture, &mut start, &mut end);

        float_into_buffer(&mut buffer, self.heights[0], &mut start, &mut end);
        float_into_buffer(&mut buffer, self.heights[1], &mut start, &mut end);
        float_into_buffer(&mut buffer, self.heights[2], &mut start, &mut end);

        float_into_buffer(&mut buffer, self.pathness[0], &mut start, &mut end);
        float_into_buffer(&mut buffer, self.pathness[1], &mut start, &mut end);
        float_into_buffer(&mut buffer, self.pathness[2], &mut start, &mut end);

        u32_into_buffer(&mut buffer, self.health, &mut start, &mut end);
        u32_into_buffer(&mut buffer, self.constitution, &mut start, &mut end);

        buffer
    }

    pub fn from_bytes(data: &[u8;MAP_ENTITY_SIZE]) -> Self {
        let mut start : usize;
        let end : usize;

        start = 0;
        end = start + 6;

        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let id = TetrahedronId::from_bytes(&buffer);
        start = end;

        let last_update = decode_u32(data, &mut start);
        let prop = decode_u32(data, &mut start);
        let faction = data[start];
        start += 1;
        let level = data[start];
        start += 1;

        let temperature = decode_float(data, &mut start);
        let moisture = decode_float(data, &mut start);

        let heights = [
            decode_float(data, &mut start),
            decode_float(data, &mut start),
            decode_float(data, &mut start)
        ];

        let pathness = [
            decode_float(data, &mut start),
            decode_float(data, &mut start),
            decode_float(data, &mut start)
        ];

        let health = decode_u32(data, &mut start);
        let constitution = decode_u32(data, &mut start);

        MapEntity {object_id: None, id, last_update, prop, faction, level, temperature, moisture, heights, pathness, health, constitution}
    }
}


fn float_into_buffer(buffer : &mut [u8;MAP_ENTITY_SIZE], data: f32, start : &mut usize, end: &mut usize)
{
    *end = *end + 4;
    let bytes = f32::to_le_bytes(data);
    buffer[*start..*end].copy_from_slice(&bytes);
    *start = *end;
}

fn u32_into_buffer(buffer : &mut [u8;MAP_ENTITY_SIZE], data: u32, start : &mut usize, end: &mut usize)
{
    *end = *end + 4;
    let bytes = u32::to_le_bytes(data);
    buffer[*start..*end].copy_from_slice(&bytes);
    *start = *end;
}

pub fn decode_float(buffer: &[u8;MAP_ENTITY_SIZE], start: &mut usize) -> f32
{
    let end = *start + 4;
    let decoded_float = f32::from_le_bytes(buffer[*start..end].try_into().unwrap());
    *start = end;
    decoded_float
}

pub fn decode_u32(buffer: &[u8;MAP_ENTITY_SIZE], start: &mut usize) -> u32
{
    let end = *start + 4;
    let decoded_float = u32::from_le_bytes(buffer[*start..end].try_into().unwrap());
    *start = end;
    decoded_float
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_map_entity()
    {

        let entity = MapEntity{
            object_id: None,
            id: TetrahedronId::from_string("a00001"),
            last_update: 1000,
            prop: 10,
            faction: 0,
            level:1,
            temperature: 1.2,
            moisture: 0.2,
            heights: [0.2,1.0,2.2],
            pathness: [1.2,1.1,1.5],
            health: 14,
            constitution: 100,
        };

        let encoded = entity.to_bytes();

        let decoded_tile = MapEntity::from_bytes(&encoded);
        println!("{:?}", decoded_tile);
        assert_eq!(decoded_tile,entity);
    }
}