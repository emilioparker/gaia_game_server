use crate::chat::chat_entry::{CHAT_ENTRY_SIZE, ChatEntry};
use crate::real_time_service::DataType;


use std::io::prelude::*;
use std::time::SystemTime;
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub fn create_data_packets(faction: u8, data : &Vec<ChatEntry>, packet_number : &mut u64) -> Vec<(u64, u8, Vec<u8>)> 
{
    *packet_number = 0u64;
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
    for chat_entry in data
    {
        let required_space = CHAT_ENTRY_SIZE as u32 + 1;

        if stored_bytes + required_space > 5000 // 1 byte for protocol, 8 bytes for the sequence number 
        {
            buffer[start] = DataType::NoData as u8;

            encoder.write_all(buffer.as_slice()).unwrap();
            let compressed_bytes = encoder.reset(Vec::new()).unwrap();
            // println!("compressed {} vs normal {}", compressed_bytes.len(), buffer.len());
            packets.push((0, faction, compressed_bytes)); // this is a copy!

            start = 1;
            stored_states = 0;
            stored_bytes = 0;

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

        buffer[start] = DataType::ChatMessage as u8;
        start += 1;

        let message_bytes = chat_entry.to_bytes(); //63
        let next = start + CHAT_ENTRY_SIZE;
        buffer[start..next].copy_from_slice(&message_bytes);
        stored_bytes = stored_bytes +  CHAT_ENTRY_SIZE as u32 + 1;
        stored_states = stored_states + 1;
        start = next;
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
        packets.push((0, faction, compressed_bytes)); // this is a copy!
    }

    // let all_data : Vec<u8> = packets.iter().flat_map(|d| d.clone()).collect();

    packets
}