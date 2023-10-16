use crate::{SERVER_STATE_SIZE, ServerState};
use crate::character::character_attack::CHARACTER_ATTACK_SIZE;
use crate::character::character_entity::CHARACTER_ENTITY_SIZE;
use crate::character::character_presentation::CHARACTER_PRESENTATION_SIZE;
use crate::character::character_reward::{CHARACTER_REWARD_SIZE, self};
use crate::chat::chat_entry::CHAT_ENTRY_SIZE;
use crate::map::map_entity::{MAP_ENTITY_SIZE, MapEntity};
use crate::map::tile_attack::TILE_ATTACK_SIZE;
use crate::real_time_service::DataType;
use crate::real_time_service::client_handler::StateUpdate;
use crate::tower::tower_entity::TOWER_ENTITY_SIZE;


use std::io::prelude::*;
use std::time::SystemTime;
use flate2::Compression;
use flate2::write::ZlibEncoder;


pub fn create_data_packets(data : Vec<StateUpdate>, packet_number : &mut u64) -> Vec<(u64, u8, Vec<u8>)> 
{
    *packet_number += 1u64;
    // println!("{packet_number} -A");

    let mut buffer = [0u8; 5000];
    let mut start: usize = 1;
    buffer[0] = crate::protocols::Protocol::GlobalState as u8;

    let packet_number_bytes = u64::to_le_bytes(*packet_number); // 8 bytes

    let end: usize = start + 8;
    buffer[start..end].copy_from_slice(&packet_number_bytes);
    start = end;

    let result = std::time::SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    let current_time = result.ok().map(|d| d.as_secs() as u32);
    let current_time_bytes = u32::to_le_bytes(current_time.unwrap()); // 4 bytes
 
    let end: usize = start + 4;
    buffer[start..end].copy_from_slice(&current_time_bytes);
    start = end;

    let mut stored_bytes:u32 = 0;
    let mut stored_states:u8 = 0;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));


    let mut packets = Vec::<(u64, u8, Vec<u8>)>::new();
    // this is interesting, this list is shared between threads/clients but since I only read it, it is fine.

    // println!("data to send {}" , data.len());
    for state_update in data.iter()
    {
        let required_space = match state_update
        {
            StateUpdate::PlayerState(_) => CHARACTER_ENTITY_SIZE as u32 + 1,
            StateUpdate::TileState(_) => MAP_ENTITY_SIZE as u32 + 1,
            StateUpdate::PlayerGreetings(_) => CHARACTER_PRESENTATION_SIZE as u32 + 1,
            StateUpdate::PlayerAttackState(_) => CHARACTER_ATTACK_SIZE as u32 + 1,
            StateUpdate::Rewards(_) =>CHARACTER_REWARD_SIZE as u32 + 1,
            StateUpdate::TileAttackState(_) =>TILE_ATTACK_SIZE as u32 + 1,
            StateUpdate::TowerState(_) => TOWER_ENTITY_SIZE as u32 + 1,
            StateUpdate::ChatMessage(_) => CHAT_ENTRY_SIZE as u32 + 1,
            StateUpdate::ServerStatus(_) => SERVER_STATE_SIZE as u32 + 1,
        };

        if stored_bytes + required_space > 5000 // 1 byte for protocol, 8 bytes for the sequence number 
        {
            buffer[start] = DataType::NoData as u8;

            encoder.write_all(buffer.as_slice()).unwrap();
            let compressed_bytes = encoder.reset(Vec::new()).unwrap();
            // println!("compressed {} vs normal {}", compressed_bytes.len(), buffer.len());
            packets.push((*packet_number, 0, compressed_bytes)); // this is a copy!

            start = 1;
            stored_states = 0;
            stored_bytes = 0;

            //a new packet with a new sequence number
            *packet_number += 1u64;
            println!("{packet_number} -B");
            let end: usize = start + 8;
            let packet_number_bytes = u64::to_le_bytes(*packet_number); // 8 bytes
            buffer[start..end].copy_from_slice(&packet_number_bytes);
            start = end;

            let result = std::time::SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
            let current_time = result.ok().map(|d| d.as_secs() as u32);
            let current_time_bytes = u32::to_le_bytes(current_time.unwrap()); // 4 bytes
        
            let end: usize = start + 4;
            buffer[start..end].copy_from_slice(&current_time_bytes);
            start = end;
        }

        match state_update{
            StateUpdate::PlayerState(player_state) => 
            {
                
                buffer[start] = DataType::PlayerState as u8;
                start += 1;

                let player_state_bytes = player_state.to_bytes(); //44
                let next = start + CHARACTER_ENTITY_SIZE;
                buffer[start..next].copy_from_slice(&player_state_bytes);
                stored_bytes = stored_bytes + CHARACTER_ENTITY_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::TileState(tile_state) => 
            {
                buffer[start] = DataType::TileState as u8;
                start += 1;

                let tile_state_bytes = tile_state.to_bytes();
                let next = start + MapEntity::get_size() as usize;
                buffer[start..next].copy_from_slice(&tile_state_bytes);
                stored_bytes = stored_bytes + MapEntity::get_size() as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            }
            StateUpdate::PlayerGreetings(presentation) => 
            {
                buffer[start] = DataType::PlayerPresentation as u8;
                start += 1;

                let presentation_bytes = presentation.to_bytes(); //28
                let next = start + CHARACTER_PRESENTATION_SIZE;
                buffer[start..next].copy_from_slice(&presentation_bytes);
                stored_bytes = stored_bytes + CHARACTER_PRESENTATION_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::PlayerAttackState(player_attack) => 
            {
                buffer[start] = DataType::PlayerAttack as u8;
                start += 1;

                let attack_bytes = player_attack.to_bytes(); //24
                let next = start + CHARACTER_ATTACK_SIZE;
                buffer[start..next].copy_from_slice(&attack_bytes);
                stored_bytes = stored_bytes + CHARACTER_ATTACK_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::Rewards(player_reward) => 
            {
                buffer[start] = DataType::PlayerReward as u8; // 30
                start += 1;

                let reward_bytes = player_reward.to_bytes(); //16 bytes
                let next = start + character_reward::CHARACTER_REWARD_SIZE;
                buffer[start..next].copy_from_slice(&reward_bytes);
                stored_bytes = stored_bytes + character_reward::CHARACTER_REWARD_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::TileAttackState(tile_attack) => 
            {
                buffer[start] = DataType::TileAttack as u8;
                start += 1;

                let attack_bytes = tile_attack.to_bytes(); //22
                let next = start + TILE_ATTACK_SIZE;
                buffer[start..next].copy_from_slice(&attack_bytes);
                stored_bytes = stored_bytes + TILE_ATTACK_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::TowerState(tower_entity) => 
            {
                buffer[start] = DataType::TowerState as u8;
                start += 1;

                let tower_bytes = tower_entity.to_bytes(); //63
                let next = start + TOWER_ENTITY_SIZE;
                buffer[start..next].copy_from_slice(&tower_bytes);
                stored_bytes = stored_bytes +  TOWER_ENTITY_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::ChatMessage(message) => 
            {
                buffer[start] = DataType::ChatMessage as u8;
                start += 1;

                let message_bytes = message.to_bytes(); //63
                let next = start + CHAT_ENTRY_SIZE;
                buffer[start..next].copy_from_slice(&message_bytes);
                stored_bytes = stored_bytes +  CHAT_ENTRY_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::ServerStatus(status) =>
            {
                // println!(" status {:?}", status);
                buffer[start] = DataType::ServerStatus as u8;
                start += 1;

                let message_bytes = ServerState::stats_to_bytes(status); //20
                let next = start + SERVER_STATE_SIZE;
                buffer[start..next].copy_from_slice(&message_bytes);
                stored_bytes = stored_bytes + SERVER_STATE_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
        }

    }

    if stored_states > 0
    {
        buffer[start] = DataType::NoData as u8;
        let trimmed_buffer = &buffer[..(start + 1)];
        
        encoder.write_all(trimmed_buffer).unwrap();
        // encoder.write_all(buffer.as_slice()).unwrap();
        let compressed_bytes = encoder.reset(Vec::new()).unwrap();
        // println!("compressed {} vs normal {}", compressed_bytes.len(), trimmed_buffer.len());


        // let data : &[u8] = &compressed_bytes;
        // let mut decoder = ZlibDecoder::new(data);

        // let decoded_data_result :  Result<Vec<u8>, _> = decoder.bytes().collect();
        // let decoded_data = decoded_data_result.unwrap();
        // let decoded_data_array : &[u8] = &decoded_data;

        // println!("data:");
        // println!("{:#04X?}", buffer);

        // println!("decoded data: {}", (buffer == *decoded_data_array));
        packets.push((*packet_number, 0, compressed_bytes)); // this is a copy!
    }

    // let all_data : Vec<u8> = packets.iter().flat_map(|d| d.clone()).collect();

    packets
}