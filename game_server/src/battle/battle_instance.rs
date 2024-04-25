use std::collections::{HashMap, HashSet};

use rand::seq::index;
use tokio::time::error::Elapsed;

use crate::map::tetrahedron_id::TetrahedronId;

pub const BATTLE_INSTANCE_SIZE: usize = 18;

#[derive(Debug)]
pub struct BattleInstance
{
    pub target_tile_id: TetrahedronId, // 6 bytes
    pub version: u8,
    pub turn: u8, // 4 bytes
    pub turn_time : u32, // if everyone participating has attacked, we move on.
    pub participants_log: u8, // 1 bytes
    pub turn_log: u8, // 1 bytes
    pub participants: HashMap<u16, u8>,
    pub last_enemy_card_used: u32
}

impl Clone for BattleInstance 
{
    fn clone(&self) -> Self 
    {
        Self { 
            target_tile_id: self.target_tile_id.clone(),
            version: self.version.clone(),
            turn: self.turn.clone(),
            turn_time: self.turn_time.clone(),
            participants_log: self.participants_log.clone(),
            turn_log: self.turn_log.clone(),
            participants: HashMap::with_capacity(0),
            last_enemy_card_used: 0
        }
    }
}

impl BattleInstance 
{
    pub fn new(id : TetrahedronId, time :u32) -> Self
    {
        BattleInstance
        {
            target_tile_id: id,
            version: 0,
            turn: 1, // 0 means finished
            turn_time: time,
            participants_log: 0,
            turn_log: 0,
            participants: HashMap::new(),
            last_enemy_card_used: 0,
        }
    }

    pub fn finish(&mut self)
    {
        self.version = 0;
        self.turn = 0;
        self.turn_time = 0;
        self.participants_log = 0;
        self.turn_log = 0;
        self.participants.clear();
        self.last_enemy_card_used = 0;
    }

    pub fn reset(&mut self, time :u32)
    {
        self.finish();
        self.turn_time = time;
        self.turn = 1;

    }

    pub fn join_battle(&mut self, player_id : u16) -> Option<u8>
    {
        let index = u8::BITS - self.participants_log.leading_zeros();

        if let Some(result) =  self.participants.get(&player_id)
        {
            return Some(*result);
        }
        else
        {
            // doesn't matter if it is invalid.
            self.participants.insert(player_id, index as u8);
        }

        if index == 8 // this means it is full
        {
            return None;
        }

        // we need to get the next available index.
        self.participants_log = self.participants_log | (1 << index);
        // if there are other players, we assume you participated in this turn.
        // if self.participants_log >  u8::pow(2,index)
        self.turn_log = self.turn_log | (1 << index); 
        self.version += 1;
        Some(index as u8)
    }

    fn everyone_participated(&self) -> bool
    {
        self.participants_log & self.turn_log == self.participants_log
    }

    fn has_already_participated(&self, participant_id :u8) -> bool
    {
        let state =  (self.turn_log >> participant_id) & 1;
        println!("check if player {participant_id} has already participated in {:b} result {state}", self.turn_log);
        state == 1
    }

    fn register_disconnected_players(&mut self)
    {
        println!("checking disconnected with {} {} " , self.participants_log, self.turn_log);
        self.participants_log = self.turn_log & self.participants_log;
    }

    pub fn play_turn(&mut self, index:u8, player_id: u16, current_time_in_seconds : u32) -> bool
    {
        if let Some(participant_id) = self.participants.get(&player_id)
        {
            if *participant_id != index 
            {
                return false;
            }
        }
        else
        {
            return false;
        }

        let state =  (self.participants_log >> index) & 1;
        if state == 0 
        {
            println!("not a participant {}" , self.participants_log);
            return false;
        }

        // this means we should move on to the next turn
        if self.turn_time < current_time_in_seconds || self.everyone_participated()
        {
            self.register_disconnected_players();
            self.turn_time = current_time_in_seconds + 5;
            self.turn += 1;
            self.turn_log = 0;
        }

        if self.has_already_participated(index)
        {
            println!("player has already participated on this turn {index}");
            false
        }
        else
        {
            println!("turn log {}", self.turn_log );
            println!("index {}", index);
            println!("new value to add {}", (1 << index));
            // we set it to 1
            self.turn_log = self.turn_log | (1 << index);
            println!("player turn registered {index} result: {}", self.turn_log);
            self.version += 1;
            true
        }
    }
    
    // used by the test_client ignores the protocol byte.
    pub fn to_bytes(&self) -> [u8;BATTLE_INSTANCE_SIZE] 
    {
        let mut buffer = [0u8;BATTLE_INSTANCE_SIZE];
        let mut start : usize;
        let mut end : usize;

        start = 0;
        end = start + 6;
        let tile_id = self.target_tile_id.to_bytes(); // 6 bytes
        buffer[start..end].copy_from_slice(&tile_id);

        start = end;
        end = start + 1;
        buffer[start] = self.version;

        start = end;
        end = start + 1;
        buffer[start] = self.turn;

        start = end;
        end = start + 4; 
        let turn_time_bytes = u32::to_le_bytes(self.turn_time); // 4 bytes
        buffer[start..end].copy_from_slice(&turn_time_bytes);

        start = end;
        end = start + 1;
        buffer[start] = self.participants_log;

        start = end;
        end = start + 1;
        buffer[start] = self.turn_log;

        start = end;
        end = start + 4; 
        let last_enemy_card_bytes = u32::to_le_bytes(self.last_enemy_card_used); // 4 bytes
        buffer[start..end].copy_from_slice(&last_enemy_card_bytes);

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join()
    {
        let mut battle_instance = BattleInstance
        {
            target_tile_id: TetrahedronId::from_string("a00"),
            version :0,
            turn: 0,
            turn_time: 0,
            participants_log: 0,
            turn_log: 0,
            participants: HashMap::new(),
            last_enemy_card_used: 0,
        };


        let id = battle_instance.join_battle(30);
        assert_eq!(id.unwrap(), 0);

        for i in 0..7
        {
            let id_2 = battle_instance.join_battle(31 + i);
            println!("new player on {id_2:?}");
            assert_eq!(id_2.unwrap() as u16, i + 1);
        }

        let already_joined = battle_instance.join_battle(30);
        assert_eq!(already_joined, Some(0));

        let invalid = battle_instance.join_battle(15);
        assert_eq!(invalid, None);


        println!("-----------Test turn 0");
        let mut current_time = 0;
        let result = battle_instance.play_turn(id.unwrap(),30, current_time);
        println!("log: {:b}", battle_instance.turn_log);
        assert!(result);
        assert!(battle_instance.turn_log == 0b11111111);

        println!("---------- Test turn 1");
        let result = battle_instance.play_turn(0, 30, current_time);
        assert!(result);
        println!("{}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b00000001);

        println!("-----------Test turn 2");
        let result = battle_instance.play_turn(3, 33, current_time);
        assert!(result);
        println!("{}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b00001001);

        println!("-----------Test turn 3");
        let result = battle_instance.play_turn(7, 37, current_time);
        assert!(result);
        println!("{}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b10001001);

        println!("some one doesn't want to wait");
        let result = battle_instance.play_turn(0, 30, current_time);
        assert!(result == false);
        println!("{}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b10001001);

        println!("a lot of time passed and no one particpated");
        current_time += 6;
        let result = battle_instance.play_turn(0, 30, current_time);
        assert!(result);
        assert!(battle_instance.turn == 2);
        println!("log {}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b00000001);

        println!("a player that left cannot particpate");
        let result = battle_instance.play_turn(4,34, current_time);
        assert!(result == false);
        assert!(battle_instance.turn == 2);
        println!("log {}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b00000001);

        println!("a player that left cannot particpate");
        let result = battle_instance.play_turn(7,37, current_time);
        assert!(result);
        assert!(battle_instance.turn == 2);
        println!("log {}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b10000001);

        println!("last player");
        let result = battle_instance.play_turn(3,33, current_time);
        assert!(result);
        assert!(battle_instance.turn == 2);
        println!("log {}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b10001001);

        println!("that same player moves on, no need to wait");
        current_time += 1;
        let result = battle_instance.play_turn(3,33, current_time);
        assert!(result);
        assert!(battle_instance.turn == 3);
        println!("log {}", battle_instance.turn_log);
        assert!(battle_instance.turn_log == 0b00001000);

    }
}