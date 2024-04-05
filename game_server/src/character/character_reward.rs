pub const CHARACTER_REWARD_SIZE: usize = 12;

#[derive(Debug, Clone, PartialEq)]
pub struct CharacterReward {
    pub player_id: u16, // 2 bytes
    pub item_id: u32, // 4 bytes
    pub amount: u16, // 2 bytes
    pub inventory_hash: u32 // 4 bytes
}

impl CharacterReward {
    // used by the test_client ignores the protocol byte.
    pub fn to_bytes(&self) -> [u8;CHARACTER_REWARD_SIZE] {
        let mut buffer = [0u8; CHARACTER_REWARD_SIZE];

        let mut start : usize = 0;
        let mut end : usize = 2;

        let player_id_bytes = u16::to_le_bytes(self.player_id); // 2 bytes
        buffer[start..end].copy_from_slice(&player_id_bytes);
        start = end;
        end = start + 4;
        u32_into_buffer(&mut buffer,self.item_id, &mut start, end);
        start = end;
        end = start + 2;
        let amount_bytes = u16::to_le_bytes(self.amount); // 2 bytes
        buffer[start..end].copy_from_slice(&amount_bytes);
        start = end;
        end = start + 4;
        let amount_bytes = u32::to_le_bytes(self.inventory_hash); // 4 bytes
        buffer[start..end].copy_from_slice(&amount_bytes);
        buffer
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        let mut start = 0;
        let mut end = start + 2;

        let player_id = u16::from_le_bytes(data[start..end].try_into().unwrap());
        start = end;

        end = start + 4;
        let item_id = decode_u32(data, &mut start, end);
        start = end;

        end = start + 2;
        let amount = u16::from_le_bytes(data[start..end].try_into().unwrap());
        start = end;

        end = start + 4;
        let inventory_hash = u32::from_le_bytes(data[start..end].try_into().unwrap());
        //start = end;

        CharacterReward { player_id, item_id, amount, inventory_hash}
    }
}

pub fn decode_u32(buffer: &[u8], start: &mut usize, end: usize) -> u32
{
    let decoded_u32 = u32::from_le_bytes(buffer[*start..(*start + 4)].try_into().unwrap());
    *start = end;
    decoded_u32
}

fn u32_into_buffer(buffer : &mut [u8], data: u32, start : &mut usize, end: usize)
{
    let bytes = u32::to_le_bytes(data);
    buffer[*start..end].copy_from_slice(&bytes);
    *start = end;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_map_entity()
    {

        let reward = CharacterReward
        {
            player_id: 12300,
            item_id: 34,
            amount: 101,
            inventory_hash: 1,
        };

        let encoded = reward.to_bytes();
        println!("encoded size {}", encoded.len());

        let decoded_reward = CharacterReward::from_bytes(&encoded);
        assert_eq!(decoded_reward,reward);
    }
}