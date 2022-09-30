
        // public static int EncodeTileId(TetrahedronId id, int offset, byte[] tileBytes)
        // {
        //     byte[] subID = BitConverter.GetBytes(id.Id); // 4 bytes
        //     tileBytes[offset + 0] = id.Area;
        //     tileBytes[offset + 1] = subID[0];
        //     tileBytes[offset + 2] = subID[1];
        //     tileBytes[offset + 3] = subID[2];
        //     tileBytes[offset + 4] = subID[3];
        //     tileBytes[offset + 5] = id.Lod;
        //     return offset + 6;
        // }

        // public static (int, TetrahedronId) DecodeTileId(int offset, byte[] input)
        // {
        //     byte area = input[offset];
        //     uint subId = BitConverter.ToUInt32(input, offset + 1);
        //     byte lod = input[offset + 5];
        //     return (offset + 6, new TetrahedronId{Area = area, Id = subId, Lod = lod});
        // }
        // public byte Area;
        // public uint Id;
        // public byte Lod;

#[derive(Debug)]
pub struct TetrahedronId {
    pub area : u8,
    pub id : u32,
    pub lod : u8
}

impl TetrahedronId {
    pub fn to_bytes(&self) -> [u8;6] {
        let mut buffer = [0u8; 6];
        let mut start : usize = 0;
        let mut end : usize = 0;

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
}
