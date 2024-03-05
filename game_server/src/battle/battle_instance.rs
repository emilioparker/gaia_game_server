use rand::seq::index;
use tokio::time::error::Elapsed;

use crate::map::tetrahedron_id::TetrahedronId;

pub const BATTLE_INSTANCE_SIZE: usize = 18;

#[derive(Debug, Clone)]
pub struct BattleInstance
{
    pub target_tile_id: TetrahedronId, // 6 bytes
    pub turn: u8, // 4 bytes
    pub turn_time : u32, // if everyone participating has attacked, we move on.
    pub participants_log: u8, // 1 bytes
    pub turn_log: u8, // 1 bytes
}

impl BattleInstance 
{
    pub fn join_battle(&mut self) -> u8
    {
        let index = u8::BITS - self.participants_log.leading_zeros();
        if index == 8 // this means it is full
        {
            return 8;
        }

        // we need to get the next available index.
        self.participants_log = self.participants_log | (1 << index);
        self.turn_log = self.turn_log | (1 << index); // ya participaste en lo primero, sea lo que sea.

        index as u8
    }

    fn everyone_participated(&self) -> bool
    {
        self.participants_log & self.turn_log == self.participants_log
    }

    fn register_disconnected_players(&mut self)
    {

        println!("checking disconnected with {} {} " , self.participants_log, self.turn_log);
        self.participants_log = self.turn_log & self.participants_log;
    }

    pub fn play_turn(&mut self, index:u8, current_time_in_seconds : u32, expected_turn : u8) -> bool
    {
        let state =  (self.participants_log >> index) & 1;
        if state == 0 
        {
            println!("not a participant {}" , self.participants_log);
            return false;
        }

        // this means we should move on to the next turn
        if self.turn_time < current_time_in_seconds && (self.turn + 1) == expected_turn
            || (self.turn + 1) == expected_turn && self.everyone_participated()
        {
            self.register_disconnected_players();
            self.turn_time = current_time_in_seconds;
            self.turn += 1;
            self.turn_log = 0;
        }

        if expected_turn == self.turn
        {
            println!("turn log {}", self.turn_log );
            println!("index {}", index);
            println!("new value to add {}", (1 << index));
            // we set it to 1
            self.turn_log = self.turn_log | (1 << index);
            println!("player turn registered {index} result: {}", self.turn_log);
        }
        else 
        {
            println!("wrong expected turn {}, current is {}", expected_turn, self.turn);
        }

        expected_turn == self.turn

    }
    

    // used by the test_client ignores the protocol byte.
    pub fn to_bytes(&self) -> [u8;BATTLE_INSTANCE_SIZE] {
        todo!();
    }

    pub fn from_bytes(data: &[u8;508]) -> Self {
        todo!();
    }
}

pub fn decode_u32(buffer: &[u8;508], start: &mut usize, end: usize) -> u32
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
    fn test_join()
    {
        let mut battle_instance = BattleInstance{
            target_tile_id: TetrahedronId::from_string("a00"),
            turn: 0,
            turn_time: 0,
            participants_log: 0,
            turn_log: 0,
        };


        let id = battle_instance.join_battle();
        assert_eq!(id, 0);

        for i in 0..7
        {
            let id_2 = battle_instance.join_battle();
            println!("new player on {id_2}");
            assert_eq!(id_2, i + 1);
        }

        let invalid = battle_instance.join_battle();
        assert_eq!(invalid, 8);

        let invalid = battle_instance.join_battle();
        assert_eq!(invalid, 8);


        println!("-----------Test turn 0");
        let mut current_time = 0;
        let result = battle_instance.play_turn(id, current_time, 0);
        println!("log: {}", battle_instance.turn_log);
        assert!(result);
        assert!(battle_instance.turn_log == 0b11111111);

        println!("---------- Test turn 1");
        let result = battle_instance.play_turn(0, current_time, 1);
        assert!(result);
        println!("{}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b00000001);

        println!("-----------Test turn 2");
        let result = battle_instance.play_turn(3, current_time, 1);
        assert!(result);
        println!("{}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b00001001);

        println!("-----------Test turn 3");
        let result = battle_instance.play_turn(7, current_time, 1);
        assert!(result);
        println!("{}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b10001001);

        println!("some one doesn't want to wait");
        let result = battle_instance.play_turn(0, current_time, 2);
        assert!(result == false);
        println!("{}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b10001001);

        println!("a lot of time passed and no one particpated");
        current_time += 1;
        let result = battle_instance.play_turn(0, current_time, 2);
        assert!(result);
        assert!(battle_instance.turn == 2);
        println!("log {}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b00000001);

        println!("a player that left cannot particpate");
        let result = battle_instance.play_turn(4, current_time, 2);
        assert!(result == false);
        assert!(battle_instance.turn == 2);
        println!("log {}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b00000001);

        println!("a player that left cannot particpate");
        let result = battle_instance.play_turn(7, current_time, 2);
        assert!(result);
        assert!(battle_instance.turn == 2);
        println!("log {}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b10000001);

        println!("last player");
        let result = battle_instance.play_turn(3, current_time, 2);
        assert!(result);
        assert!(battle_instance.turn == 2);
        println!("log {}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b10001001);

        println!("that same player moves on, no need to wait");
        current_time += 1;
        let result = battle_instance.play_turn(3, current_time, 3);
        assert!(result);
        assert!(battle_instance.turn == 3);
        println!("log {}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b00001000);

    }
}