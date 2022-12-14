use super::tetrahedron_id::TetrahedronId;

#[derive(Debug, Clone, PartialEq)]
pub struct MapEntity { // 69 bytes
    pub id : TetrahedronId, // 6 bytes
    pub last_update: u32, // 4 bytes
    pub health:u32, // 4 bytes
    pub prop: u32, // 4 bytes
    pub heat:u8,
    pub moisture:u8,
    pub biome:u8,
    pub heights : [u32;3], // 12 bytes
    pub normal_a : [f32;3], // 12 bytes
    pub normal_b : [f32;3], // 12 bytes
    pub normal_c : [f32;3], // 12 bytes
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
    pub fn to_bytes(&self) -> [u8;69] {
        let mut buffer = [0u8;69];
        let mut start : usize;
        let mut end : usize;

        start = 0;
        end = start + 6;
        let tile_id = self.id.to_bytes(); // 6 bytes
        buffer[start..end].copy_from_slice(&tile_id);
        start = end;

        u32_into_buffer(&mut buffer, self.last_update, &mut start, &mut end);
        u32_into_buffer(&mut buffer, self.health, &mut start, &mut end);
        u32_into_buffer(&mut buffer, self.prop, &mut start, &mut end);

        buffer[start] = self.heat;
        buffer[start + 1] = self.moisture;
        buffer[start + 2] = self.biome;
        start = start + 3;
        end = start;

        u32_into_buffer(&mut buffer, self.heights[0], &mut start, &mut end);
        u32_into_buffer(&mut buffer, self.heights[1], &mut start, &mut end);
        u32_into_buffer(&mut buffer, self.heights[2], &mut start, &mut end);

        float_into_buffer(&mut buffer, self.normal_a[0], &mut start, &mut end);
        float_into_buffer(&mut buffer, self.normal_a[1], &mut start, &mut end);
        float_into_buffer(&mut buffer, self.normal_a[2], &mut start, &mut end);

        float_into_buffer(&mut buffer, self.normal_b[0], &mut start, &mut end);
        float_into_buffer(&mut buffer, self.normal_b[1], &mut start, &mut end);
        float_into_buffer(&mut buffer, self.normal_b[2], &mut start, &mut end);

        float_into_buffer(&mut buffer, self.normal_c[0], &mut start, &mut end);
        float_into_buffer(&mut buffer, self.normal_c[1], &mut start, &mut end);
        float_into_buffer(&mut buffer, self.normal_c[2], &mut start, &mut end);

        buffer
    }

    pub fn from_bytes(data: &[u8;69]) -> Self {
        let mut start : usize;
        let end : usize;

        start = 0;
        end = start + 6;

        let mut buffer = [0u8;6];
        buffer.copy_from_slice(&data[start..end]);
        let id = TetrahedronId::from_bytes(&buffer);
        start = end;

        let last_update = decode_u32(data, &mut start);
        let health = decode_u32(data, &mut start);
        let prop = decode_u32(data, &mut start);

        let heat = data[start];
        start += 1;
        let moisture = data[start];
        start += 1;
        let biome = data[start];
        start += 1;

        let heights = [
            decode_u32(data, &mut start),
            decode_u32(data, &mut start),
            decode_u32(data, &mut start)
        ];

        let normal_a = [
            decode_float(data, &mut start),
            decode_float(data, &mut start),
            decode_float(data, &mut start)
        ];
        let normal_b = [
            decode_float(data, &mut start),
            decode_float(data, &mut start),
            decode_float(data, &mut start)
        ];
        let normal_c = [
            decode_float(data, &mut start),
            decode_float(data, &mut start),
            decode_float(data, &mut start)
        ];

        MapEntity { id, last_update, health, prop, heat, moisture, biome, heights, normal_a, normal_b, normal_c }
    }
}


impl MapCommand {
    pub fn from_bytes(data: &[u8;508]) -> Self {
        let mut start : usize;
        let mut end : usize;

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


fn float_into_buffer(buffer : &mut [u8;69], data: f32, start : &mut usize, end: &mut usize)
{
    *end = *end + 4;
    let bytes = f32::to_le_bytes(data);
    buffer[*start..*end].copy_from_slice(&bytes);
    *start = *end;
}

fn u32_into_buffer(buffer : &mut [u8;69], data: u32, start : &mut usize, end: &mut usize)
{
    *end = *end + 4;
    let bytes = u32::to_le_bytes(data);
    buffer[*start..*end].copy_from_slice(&bytes);
    *start = *end;
}

pub fn decode_float(buffer: &[u8;69], start: &mut usize) -> f32
{
    let end = *start + 4;
    let decoded_float = f32::from_le_bytes(buffer[*start..end].try_into().unwrap());
    *start = end;
    decoded_float
}

pub fn decode_u32(buffer: &[u8;69], start: &mut usize) -> u32
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
            id: TetrahedronId::from_string("a00001"),
            last_update: 1000,
            health: 14,
            prop: 10,
            heat: 1,
            moisture: 2,
            biome:3,
            heights: [0,1,2],
            normal_a: [1.2,1.1,1.5],
            normal_b: [1.2,1.1,1.6],
            normal_c: [1.2,1.1,1.7],
        };

        let encoded = entity.to_bytes();

        let decoded_tile = MapEntity::from_bytes(&encoded);
        assert_eq!(decoded_tile,entity);
    }
}