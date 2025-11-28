use crate::ability_user::attack::ATTACK_SIZE;
use crate::ability_user::attack_result::ATTACK_RESULT_SIZE;
use crate::mob::mob_entity::{self, MOB_ENTITY_SIZE};
use crate::{SERVER_STATE_SIZE, ServerState};
use crate::hero::hero_entity::HERO_ENTITY_SIZE;
use crate::hero::hero_presentation::HERO_PRESENTATION_SIZE;
use crate::hero::hero_reward::{HERO_REWARD_SIZE, self};
use crate::chat::chat_entry::CHAT_ENTRY_SIZE;
use crate::map::map_entity::{MAP_ENTITY_SIZE, MapEntity};
use crate::clients_service::DataType;
use crate::clients_service::client_handler::StateUpdate;
use crate::tower::tower_entity::TOWER_ENTITY_SIZE;


use std::io::prelude::*;
use std::time::SystemTime;
use bytes::Bytes;
use flate2::Compression;
use flate2::write::ZlibEncoder;

use super::PacketsData;

pub fn init_data_packet(packets_data : &mut PacketsData)
    -> usize
{
    packets_data.packet_number += 1u64;
    // cli_log::info!("{packet_number} -A");

    let mut start: usize = 1;
    packets_data.buffer[0] = crate::protocols::Protocol::GlobalState as u8;

    let packet_number_bytes = u64::to_le_bytes(packets_data.packet_number); // 8 bytes

    let end: usize = start + 8;
    packets_data.buffer[start..end].copy_from_slice(&packet_number_bytes);
    start = end;

    let result = std::time::SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    let current_time = result.ok().map(|d| d.as_secs() as u32);
    let current_time_bytes = u32::to_le_bytes(current_time.unwrap()); // 4 bytes
 
    let end: usize = start + 4;
    packets_data.buffer[start..end].copy_from_slice(&current_time_bytes);
    start = end;

    start
}

pub fn build_data_packet(
    regions_packets_data: &mut PacketsData,
    data_type: DataType,
    chunk : &[u8],
    chunk_size: usize)
{
    if !regions_packets_data.started
    {
        regions_packets_data.offset = init_data_packet(regions_packets_data);
        regions_packets_data.game_packets_count = 0;
        regions_packets_data.started = true;
    }

    if regions_packets_data.offset + chunk_size + 1 > 5000
    {
        // this means we already have some data
        let encoded_data = encode_packet(&mut regions_packets_data.buffer, regions_packets_data.offset);
        regions_packets_data.packets.push((regions_packets_data.packet_number, 0, 0, regions_packets_data.game_packets_count, Bytes::from(encoded_data)));
        regions_packets_data.offset = init_data_packet(regions_packets_data);
        regions_packets_data.game_packets_count = 0;
    }

    add_to_data_packet(&mut regions_packets_data.buffer, &mut regions_packets_data.offset, &mut regions_packets_data.game_packets_count, data_type, chunk_size, &chunk);
}

pub fn add_to_data_packet(
    buffer : &mut [u8;5000],
    offset: &mut usize ,
    game_packets_count: &mut u32,
    data_type: DataType,
    chunk_size : usize,
    chunk : &[u8])
{
    let mut start = *offset;

    buffer[start] = data_type as u8;
    start += 1;

    let next = start + chunk_size;
    buffer[start..next].copy_from_slice(chunk);
    *offset = next;
    *game_packets_count += 1;
}

pub fn encode_packet(buffer : &mut [u8;5000], start : usize) -> Vec<u8>
{
    let mut output = Vec::new();
    let mut encoder = ZlibEncoder::new(&mut output, Compression::fast());
    buffer[start] = DataType::NoData as u8;
    let trimmed_buffer = &buffer[..(start + 1)];
    
    encoder.write_all(trimmed_buffer).unwrap();
    // encoder.write_all(buffer.as_slice()).unwrap();
    encoder.flush().unwrap();
    drop(encoder);
    output
}


pub fn create_data_packets_deprecated(data : &Vec<StateUpdate>, packet_number : &mut u64) -> Vec<(u64, u8, Vec<u8>)> 
{
    *packet_number += 1u64;
    // cli_log::info!("{packet_number} -A");

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

    // cli_log::info!("data to send {}" , data.len());
    for state_update in data.iter()
    {
        let required_space = match state_update
        {
            StateUpdate::PlayerState(_) => HERO_ENTITY_SIZE as u32 + 1,
            StateUpdate::TileState(_) => MAP_ENTITY_SIZE as u32 + 1,
            StateUpdate::PlayerGreetings(_) => HERO_PRESENTATION_SIZE as u32 + 1,
            StateUpdate::AttackState(_) => ATTACK_SIZE as u32 + 1,
            StateUpdate::Rewards(_) =>HERO_REWARD_SIZE as u32 + 1,
            StateUpdate::TowerState(_) => TOWER_ENTITY_SIZE as u32 + 1,
            StateUpdate::ChatMessage(_) => CHAT_ENTRY_SIZE as u32 + 1,
            StateUpdate::ServerStatus(_) => SERVER_STATE_SIZE as u32 + 1,
            StateUpdate::MobUpdate(_) => MOB_ENTITY_SIZE as u32 + 1,
            StateUpdate::AttackResultState(_) => ATTACK_RESULT_SIZE as u32 + 1,
        };

        if stored_bytes + required_space > 5000 // 1 byte for protocol, 8 bytes for the sequence number 
        {
            buffer[start] = DataType::NoData as u8;

            encoder.write_all(buffer.as_slice()).unwrap();
            let compressed_bytes = encoder.reset(Vec::new()).unwrap();
            // cli_log::info!("compressed {} vs normal {}", compressed_bytes.len(), buffer.len());
            packets.push((*packet_number, 0, compressed_bytes)); // this is a copy!

            start = 1;
            stored_states = 0;
            stored_bytes = 0;

            //a new packet with a new sequence number
            *packet_number += 1u64;
            cli_log::info!("{packet_number} -B");
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

        match state_update
        {
            StateUpdate::PlayerState(player_state) => 
            {
                
                buffer[start] = DataType::PlayerState as u8;
                start += 1;

                let player_state_bytes = player_state.to_bytes(); //44
                let next = start + HERO_ENTITY_SIZE;
                buffer[start..next].copy_from_slice(&player_state_bytes);
                stored_bytes = stored_bytes + HERO_ENTITY_SIZE as u32 + 1;
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
                let next = start + HERO_PRESENTATION_SIZE;
                buffer[start..next].copy_from_slice(&presentation_bytes);
                stored_bytes = stored_bytes + HERO_PRESENTATION_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::AttackState(player_attack) => 
            {
                buffer[start] = DataType::Attack as u8;
                start += 1;

                let attack_bytes = player_attack.to_bytes(); //24
                let next = start + ATTACK_SIZE;
                buffer[start..next].copy_from_slice(&attack_bytes);
                stored_bytes = stored_bytes + ATTACK_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::AttackResultState(details) => 
            {
                buffer[start] = DataType::AttackDetails as u8;
                start += 1;

                let details_bytes = details.to_bytes(); //24
                let next = start + ATTACK_RESULT_SIZE;
                buffer[start..next].copy_from_slice(&details_bytes);
                stored_bytes = stored_bytes + ATTACK_RESULT_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::Rewards(player_reward) => 
            {
                buffer[start] = DataType::PlayerReward as u8; // 30
                start += 1;

                let reward_bytes = player_reward.to_bytes(); //16 bytes
                let next = start + hero_reward::HERO_REWARD_SIZE;
                buffer[start..next].copy_from_slice(&reward_bytes);
                stored_bytes = stored_bytes + hero_reward::HERO_REWARD_SIZE as u32 + 1;
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
                // cli_log::info!(" status {:?}", status);
                buffer[start] = DataType::ServerStatus as u8;
                start += 1;

                let message_bytes = ServerState::stats_to_bytes(status); //20
                let next = start + SERVER_STATE_SIZE;
                buffer[start..next].copy_from_slice(&message_bytes);
                stored_bytes = stored_bytes + SERVER_STATE_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::MobUpdate(mob_instance) => 
            {
                // cli_log::info!(" status {:?}", status);
                buffer[start] = DataType::MobStatus as u8;
                start += 1;

                let message_bytes = mob_instance.to_bytes();
                let next = start + MOB_ENTITY_SIZE;
                buffer[start..next].copy_from_slice(&message_bytes);
                stored_bytes = stored_bytes + MOB_ENTITY_SIZE as u32 + 1;
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
        // cli_log::info!("compressed {} vs normal {}", compressed_bytes.len(), trimmed_buffer.len());


        // let data : &[u8] = &compressed_bytes;
        // let mut decoder = ZlibDecoder::new(data);

        // let decoded_data_result :  Result<Vec<u8>, _> = decoder.bytes().collect();
        // let decoded_data = decoded_data_result.unwrap();
        // let decoded_data_array : &[u8] = &decoded_data;

        // cli_log::info!("data:");
        // cli_log::info!("{:#04X?}", buffer);

        // cli_log::info!("decoded data: {}", (buffer == *decoded_data_array));
        packets.push((*packet_number, 0, compressed_bytes)); // this is a copy!
    }

    // let all_data : Vec<u8> = packets.iter().flat_map(|d| d.clone()).collect();

    packets
}