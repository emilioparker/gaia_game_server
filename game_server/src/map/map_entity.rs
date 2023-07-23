use bson::oid::ObjectId;

use super::tetrahedron_id::TetrahedronId;

pub const MAP_ENTITY_SIZE: usize = 76;

#[derive(Debug, Clone, PartialEq)]
pub struct MapEntity { // 76 bytes
    pub object_id : Option<ObjectId>,
    pub version: u16, // 2 bytes
    pub id : TetrahedronId, // 6 bytes

    // to handle who is commanding this tile with a timeout
    pub owner_id : u16, //2 bytes
    pub ownership_time : u32, // 4 bytes

    // for moving between origin and target
    pub origin_id : TetrahedronId, // 6 bytes
    pub target_id : TetrahedronId, // 6 bytes
    pub time : u32,// 4 bytes

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
            version: 1000,
            id: TetrahedronId::from_string(id),

            owner_id: 0,
            ownership_time: 0,

            origin_id: TetrahedronId::from_string(id),
            target_id: TetrahedronId::from_string(id),
            time: 0,

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
    ChangeHealth(u16,u16),
    LayFoundation(u16,u32, f32, f32, f32),
    BuildStructure(u16,u32),
    AttackWalker(u16, u32),
    SpawnMob(u32),
    MoveMob(u16,u32, TetrahedronId, f32, f32),
    ControlMob(u16, u32),
    AttackMob(u16,u16,u32),
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
        end = 0;
        u16_into_buffer(&mut buffer, self.version, &mut start, &mut end);

        end = start + 6;
        let tile_id = self.id.to_bytes(); // 6 bytes
        buffer[start..end].copy_from_slice(&tile_id);
        start = end;

        u16_into_buffer(&mut buffer, self.owner_id, &mut start, &mut end);
        u32_into_buffer(&mut buffer, self.ownership_time, &mut start, &mut end);

        end = start + 6;
        let origin_id_bytes = self.origin_id.to_bytes(); // 6 bytes
        buffer[start..end].copy_from_slice(&origin_id_bytes);
        start = end;

        end = start + 6;
        let target_id_bytes = self.target_id.to_bytes(); // 6 bytes
        buffer[start..end].copy_from_slice(&target_id_bytes);
        start = end;

        u32_into_buffer(&mut buffer, self.time, &mut start, &mut end);
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
        let mut end : usize;

        start = 0;
        let version = decode_u16(data, &mut start);

        end = start + 6;
        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let id = TetrahedronId::from_bytes(&buffer);
        start = end;

        let owner_id = decode_u16(data, &mut start);
        let ownership_time = decode_u32(data, &mut start);

        end = start + 6;
        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let origin_id = TetrahedronId::from_bytes(&buffer);
        start = end;

        end = start + 6;
        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let target_id = TetrahedronId::from_bytes(&buffer);
        start = end;

        let time = decode_u32(data, &mut start);

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

        MapEntity {
            object_id: None, 
            version,
            id,

            owner_id,
            ownership_time,

            prop,
            faction,
            level,
            temperature,
            moisture,
            heights,
            pathness,
            health,
            constitution,
            origin_id,
            target_id,
            time
        }
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

fn u16_into_buffer(buffer : &mut [u8;MAP_ENTITY_SIZE], data: u16, start : &mut usize, end: &mut usize)
{
    *end = *end + 2;
    let bytes = u16::to_le_bytes(data);
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

pub fn decode_u16(buffer: &[u8;MAP_ENTITY_SIZE], start: &mut usize) -> u16
{
    let end = *start + 2;
    let decoded_float = u16::from_le_bytes(buffer[*start..end].try_into().unwrap());
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
            version: 1000,
            id: TetrahedronId::from_string("a00001"),
            origin_id: TetrahedronId::from_string("a00001"),
            target_id: TetrahedronId::from_string("a00001"),
            time: 0,
            prop: 10,
            faction: 0,
            level:1,
            temperature: 1.2,
            moisture: 0.2,
            heights: [0.2,1.0,2.2],
            pathness: [1.2,1.1,1.5],
            health: 14,
            constitution: 100,
            owner_id: 0,
            ownership_time: 234,
        };

        let encoded = entity.to_bytes();

        let decoded_tile = MapEntity::from_bytes(&encoded);
        println!("{:?}", decoded_tile);
        assert_eq!(decoded_tile,entity);
    }
}