use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TetrahedronId {
    pub area : u8,
    pub id : u32,
    pub lod : u8
}

impl TetrahedronId {
    pub fn is_parent(&self, child : &TetrahedronId) -> bool
    {
        let factor = 4u32.pow(self.lod as u32);
        if child.id < self.id {
            return false;
        }
        let substraction = child.id - self.id;
        return self.area == child.area && self.lod < child.lod && (substraction % factor == 0)
    }

    pub fn get_parent(&self, times : usize) -> TetrahedronId
    {
        let mut current_id : u32 = self.id;
        let mut current_lod : u8 = self.lod;

        for _i in 0..times{
            let mut div_result: u32 = current_id;
            for _j in 0..(current_lod - 1){
                div_result =  (div_result as f32 / 4f32).floor() as u32;
            }

            current_id = current_id - div_result * 4u32.pow(current_lod as u32 - 1);
            current_lod = current_lod - 1;
        }

        TetrahedronId {
            area: self.area,
            id: current_id, 
            lod: current_lod,
        }
    }

    pub fn subdivide(&self, child_index : u8) -> TetrahedronId
    {
        let id = self.id  + (child_index as u32) * 4u32.pow(self.lod as u32);
        TetrahedronId { area: self.area, id, lod: self.lod + 1 }
    }

    pub fn to_bytes(&self) -> [u8;6] {
        let mut buffer = [0u8; 6];
        let start : usize;
        let end : usize;

        buffer[0] = self.area;

        start = 1;
        end = start + 4;
        let id_bytes = u32::to_le_bytes(self.id); // 4 bytes
        buffer[start..end].copy_from_slice(&id_bytes);

        buffer[end] = self.lod;

        buffer
    }

    pub fn from_bytes(data: &[u8;6]) -> Self {

        let mut start = 0;
        let mut end = start + 1;

        let area = data[start];

        start = end;
        end = start + 4;

        let id = u32::from_le_bytes(data[start..end].try_into().unwrap());
        start = end;

        let lod = data[start];
        TetrahedronId{
            area,
            id,
            lod
        }
    }

    pub fn from_string(data: &str) -> Self {
        let char_vec: Vec<char> = data.chars().collect();
        let mut area : u8 = 0;
        let mut sub_id : u32 = 0;
        for i in 0..data.len(){
            if i == 0 {
                if char_vec[0] == 'a' {area = 0};
                if char_vec[0] == 'b' {area = 1};
                if char_vec[0] == 'c' {area = 2};
                if char_vec[0] == 'd' {area = 3};
                if char_vec[0] == 'e' {area = 4};
                if char_vec[0] == 'f' {area = 5};
                if char_vec[0] == 'g' {area = 6};
                if char_vec[0] == 'h' {area = 7};
                if char_vec[0] == 'i' {area = 8};
                if char_vec[0] == 'j' {area = 9};
                if char_vec[0] == 'k' {area = 10};
                if char_vec[0] == 'l' {area = 11};
                if char_vec[0] == 'm' {area = 12};
                if char_vec[0] == 'n' {area = 13};
                if char_vec[0] == 'o' {area = 14};
                if char_vec[0] == 'p' {area = 15};
                if char_vec[0] == 'q' {area = 16};
                if char_vec[0] == 'r' {area = 17};
                if char_vec[0] == 's' {area = 18};
                if char_vec[0] == 't' {area = 19};
            }
            else {
                let num = char_vec[i].to_string().parse::<u32>().unwrap();
                sub_id += num * u32::pow(4, i as u32 - 1);
            }
        }

        TetrahedronId{
            area,
            id :sub_id,
            lod : (data.len() - 1) as u8
        }
    }
}


impl fmt::Display for TetrahedronId {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let encoded_areas : [char; 20] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't'];

        let mut alphabetic_id = "".to_string();
        let mut result = self.id;
        let end = (self.lod as i32 - 1).max(0);
        for _i in 0..end
        {
            let res = result % 4;
            result = result / 4;
            alphabetic_id.push_str(&res.to_string());
        }

        if self.lod > 0
        {
            alphabetic_id.push_str(&result.to_string());
        }

        write!(f, "{}{}", encoded_areas[self.area as usize], alphabetic_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_from_string_to_id()
    {
        let tile_human_id = "k233333313";
        let tile_id = TetrahedronId::from_string(tile_human_id);
        println!("data area:{} lod:{} id:{}", tile_id.area, tile_id.lod, tile_id.id);
        assert_eq!(tile_human_id, tile_id.to_string());

        let tile_human_id = "a233";
        let tile_id = TetrahedronId::from_string(tile_human_id);
        println!("data area:{} lod:{} id:{}", tile_id.area, tile_id.lod, tile_id.id);
        assert_eq!(tile_human_id, tile_id.to_string());

        let tile_human_id = "a";
        let tile_id = TetrahedronId::from_string(tile_human_id);
        println!("data area:{} lod:{} id:{}", tile_id.area, tile_id.lod, tile_id.id);
        assert_eq!(tile_human_id, tile_id.to_string());
    }

    #[test]
    fn get_parent_test()
    {
        let tile_human_id = "k233333313";
        let tile_id = TetrahedronId::from_string(tile_human_id);
        println!("data area:{} lod:{} id:{}", tile_id.area, tile_id.lod, tile_id.id);

        let parent = "k23333331";
        let parent_tile_id = TetrahedronId::from_string(parent);
        assert_eq!(parent_tile_id, tile_id.get_parent(1));

        let tile_human_id = "k23333";
        let tile_id = TetrahedronId::from_string(tile_human_id);

        let parent = "k2333";
        let parent_tile_id = TetrahedronId::from_string(parent);
        assert_eq!(parent_tile_id, tile_id.get_parent(1));

        let tile_human_id = "k233333333";
        let tile_id = TetrahedronId::from_string(tile_human_id);

        let parent = "k23333333";
        let parent_tile_id = TetrahedronId::from_string(parent);
        assert_eq!(parent_tile_id, tile_id.get_parent(1));

        let tile_human_id = "a220";
        let tile_id = TetrahedronId::from_string(tile_human_id);

        let parent = "a22";
        let parent_tile_id = TetrahedronId::from_string(parent);
        assert_eq!(parent_tile_id, tile_id.get_parent(1));

        let tile_human_id = "k233333313";
        let tile_id = TetrahedronId::from_string(tile_human_id);

        let parent = "k23";
        let parent_tile_id = TetrahedronId::from_string(parent);
        assert_eq!(parent_tile_id, tile_id.get_parent(7));

        let tile_human_id = "k233333313";
        let tile_id = TetrahedronId::from_string(tile_human_id);

        let parent = "k23333";
        let parent_tile_id = TetrahedronId::from_string(parent);
        assert_eq!(parent_tile_id, tile_id.get_parent(4));

    }

    #[test]
    fn convert_from_bin_to_id()
    {
        let tile_human_id = "k233333313";
        let tile_id = TetrahedronId::from_string(tile_human_id);
        let bin_data = tile_id.to_bytes();
        let tile_2 = TetrahedronId::from_bytes(&bin_data);
        println!("data area:{} lod:{} id:{}", tile_id.area, tile_id.lod, tile_id.id);
        assert_eq!(tile_id,tile_2);
    }

    #[test]
    fn is_parent()
    {
        let tile_human_id = "k23333331";
        let tile_id = TetrahedronId::from_string(tile_human_id);

        let child_human_id = "k233333310";
        let child_tile_id = TetrahedronId::from_string(child_human_id);
        assert!(tile_id.is_parent(&child_tile_id));

        let child_human_id = "k233333311";
        let child_tile_id = TetrahedronId::from_string(child_human_id);
        assert!(tile_id.is_parent(&child_tile_id));

        let child_human_id = "k233333312";
        let child_tile_id = TetrahedronId::from_string(child_human_id);
        assert!(tile_id.is_parent(&child_tile_id));

        let child_human_id = "k233333313";
        let child_tile_id = TetrahedronId::from_string(child_human_id);
        assert!(tile_id.is_parent(&child_tile_id));

        let child_human_id = "k233333323";
        let child_tile_id = TetrahedronId::from_string(child_human_id);
        assert!(tile_id.is_parent(&child_tile_id) == false);

        let child_human_id = "k2";
        let child_tile_id = TetrahedronId::from_string(child_human_id);
        assert!(tile_id.is_parent(&child_tile_id) == false);
    }
}
