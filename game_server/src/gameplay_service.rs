use std::{sync::Arc};

use crate::ServerState;
use crate::map::GameMap;
use crate::map::map_entity::{MapCommand, MapCommandInfo, MAP_ENTITY_SIZE};
use crate::player::player_attack::PlayerAttack;
use crate::player::player_entity::{InventoryItem, PLAYER_ENTITY_SIZE};
use crate::player::player_reward::{PlayerReward, self};
use crate::player::{player_command};
use crate::player::player_presentation::PlayerPresentation;
use crate::player::{player_entity::PlayerEntity, player_command::PlayerCommand};
use crate::map::{tetrahedron_id::TetrahedronId, map_entity::MapEntity};
use crate::real_time_service::client_handler::StateUpdate;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::{sync::Mutex};
use std::collections::HashMap;

use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;

pub enum DataType
{
    NoData = 25,
    PlayerState = 26,
    TileState = 27,
    PlayerPresentation = 28,
    PlayerAttack = 29,
    PlayerReward = 30,
}

pub fn start_service(
    mut rx_pc_client_game : tokio::sync::mpsc::Receiver<PlayerCommand>,
    mut rx_mc_client_game : tokio::sync::mpsc::Receiver<MapCommand>,
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    tx_bytes_game_socket: tokio::sync::mpsc::Sender<Arc<Vec<Vec<u8>>>>
) -> (Receiver<MapEntity>, Receiver<MapEntity>, Receiver<PlayerEntity>, Sender<MapCommand>) {

    let (tx_mc_webservice_gameplay, mut rx_mc_webservice_gameplay ) = tokio::sync::mpsc::channel::<MapCommand>(200);
    let (tx_me_gameplay_longterm, rx_me_gameplay_longterm ) = tokio::sync::mpsc::channel::<MapEntity>(1000);
    let (tx_me_gameplay_webservice, rx_me_gameplay_webservice) = tokio::sync::mpsc::channel::<MapEntity>(1000);
    let (tx_pe_gameplay_longterm, rx_pe_gameplay_longterm ) = tokio::sync::mpsc::channel::<PlayerEntity>(1000);

    //players
    let player_commands = HashMap::<u64,PlayerCommand>::new();
    let player_commands_mutex = Arc::new(Mutex::new(player_commands));
    let player_commands_processor_lock = player_commands_mutex.clone();
    let player_commands_agregator_lock = player_commands_mutex.clone();

    //tile commands, this means that many players might hit the same tile, but only one every 30 ms will apply, this is really cool
    //should I add luck to improve the probability of someones actions to hit the tile ?
    let tile_commands = HashMap::<TetrahedronId,MapCommand>::new();
    let tile_commands_mutex = Arc::new(Mutex::new(tile_commands));
    let tile_commands_processor_lock = tile_commands_mutex.clone();

    let tile_commands_agregator_from_client_lock = tile_commands_mutex.clone();
    let tile_commands_agregator_from_webservice_lock = tile_commands_mutex.clone();

    let mut seq = 0;

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));

    //task that will handle receiving state changes from clients and updating the global statestate.
    tokio::spawn(async move {

        loop {
            let message = rx_pc_client_game.recv().await.unwrap();

            // println!("got a player change data {}", message.player_id);
            // let mut current_time = 0;
            // let result = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
            // if let Ok(elapsed) = result {
            //     current_time = elapsed.as_secs();
            // }

            let mut data = player_commands_agregator_lock.lock().await;
            
            seq = seq + 1;
            let old = data.get(&message.player_id);
            match old {
                Some(_previous_record) => {
                    data.insert(message.player_id, message);
                }
                _ => {
                    data.insert(message.player_id, message);
                }
            }
        }
    });

    // task that gathers world changes comming from a client into a list.
    tokio::spawn(async move {

        // let mut sequence_number:u64 = 101;
        loop {
            let message = rx_mc_client_game.recv().await.unwrap();
            // println!("got a tile change data {}", message.id);
            let mut data = tile_commands_agregator_from_client_lock.lock().await;
            
            let old = data.get(&message.id);
            match old {
                Some(_previous_record) => {
                    // this command will be lost for this tile, check if it is important ??
                    data.insert(message.id.clone(), message);
                }
                _ => {
                    data.insert(message.id.clone(), message);
                }
            }

        }
    });

    // task that gathers world changes comming from web service into a list.
    tokio::spawn(async move {

        // let mut sequence_number:u64 = 101;
        loop {
            let message = rx_mc_webservice_gameplay.recv().await.unwrap();
            // println!("got a tile change data {}", message.id);
            let mut data = tile_commands_agregator_from_webservice_lock.lock().await;
            
            let old = data.get(&message.id);
            match old {
                Some(_previous_record) => {
                    // this command will be lost for this tile, check if it is important ??
                    data.insert(message.id.clone(), message);
                }
                _ => {
                    data.insert(message.id.clone(), message);
                }
            }

        }
    });

    // task that will perdiodically send dta to all clients
    tokio::spawn(async move {
        let mut packet_number = 1u64;
        loop {
            // assuming 30 fps.
            // tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            interval.tick().await;
            let mut players_summary = Vec::new();
            let mut player_attacks_summary = Vec::new();
            let mut players_presentation_summary = Vec::new();
            let mut tiles_summary : Vec<MapEntity>= Vec::new();
            let mut players_rewards_summary : Vec<PlayerReward>= Vec::new();

            let mut player_commands_data = player_commands_processor_lock.lock().await;
            let mut tile_commands_data = tile_commands_processor_lock.lock().await;
            if player_commands_data.len() <= 0  && tile_commands_data.len() <= 0{
                continue;
            }

            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let mut player_entities = map.players.lock().await;
            for item in player_commands_data.iter()
            {
                let player_command = item.1;
                let cloned_data = item.1.to_owned();

                if let Some(atomic_time) = map.active_players.get(&cloned_data.player_id){
                    atomic_time.store(current_time.as_secs(), std::sync::atomic::Ordering::Relaxed);
                }

                if player_command.action == player_command::GREET_ACTION {
                    let player_option = player_entities.get_mut(&cloned_data.player_id);
                    if let Some(player_entity) = player_option {
                        let name_with_padding = format!("{: <5}", player_entity.character_name);
                        let name_data : Vec<u32> = name_with_padding.chars().into_iter().map(|c| c as u32).collect();
                        let mut name_array = [0u32; 5];
                        name_array.clone_from_slice(&name_data.as_slice()[0..5]);
                        let player_presentation = PlayerPresentation {
                            player_id: player_entity.player_id,
                            character_name: name_array,
                        };

                        players_presentation_summary.push(player_presentation);
                    }

                }
                else if player_command.action == player_command::RESPAWN_ACTION { // respawn, we only update health for the moment
                    let player_option = player_entities.get_mut(&cloned_data.player_id);
                    if let Some(player_entity) = player_option {
                        let updated_player_entity = PlayerEntity {
                            action: player_command.action,
                            health: player_entity.constitution,
                            ..player_entity.clone()
                        };

                        *player_entity = updated_player_entity;
                        tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
                        players_summary.push(player_entity.clone());
                    }
                }
                else if player_command.action == player_command::WALK_ACTION { // respawn, we only update health for the moment
                    let player_option = player_entities.get_mut(&cloned_data.player_id);
                    if let Some(player_entity) = player_option {
                        let updated_player_entity = PlayerEntity {
                            action: player_command.action,
                            position: player_command.position,
                            second_position: player_command.second_position,
                            ..player_entity.clone()
                        };

                        *player_entity = updated_player_entity;
                        tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
                        players_summary.push(player_entity.clone());
                    }
                }
                else if player_command.action == player_command::ATTACK_ACTION { // respawn, we only update health for the moment
                    let player_option = player_entities.get_mut(&cloned_data.player_id);
                    if let Some(player_entity) = player_option {
                        let updated_player_entity = PlayerEntity {
                            action: player_command.action,
                            ..player_entity.clone()
                        };

                        *player_entity = updated_player_entity;
                        tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
                        players_summary.push(player_entity.clone());

                        if let Some(other_entity) = player_entities.get_mut(&cloned_data.other_player_id){
                            let result = other_entity.health.saturating_sub(4);
                            let updated_player_entity = PlayerEntity {
                                action: other_entity.action,
                                health: result,
                                ..other_entity.clone()
                            };

                            *other_entity = updated_player_entity;
                            tx_pe_gameplay_longterm.send(other_entity.clone()).await.unwrap();
                            players_summary.push(other_entity.clone());

                            let attack = PlayerAttack{
                                player_id: cloned_data.player_id,
                                target_player_id: cloned_data.other_player_id,
                                damage: 2,
                                skill_id: 0,
                            };
                            player_attacks_summary.push(attack);
                        }
// updating target entity, we usually substrack health I think ?
                    }
                }
                else if player_command.action == player_command::WOOD_CUT_ACTION { // respawn, we only update health for the moment
                    let player_option = player_entities.get_mut(&cloned_data.player_id);
                    if let Some(player_entity) = player_option {
                        let updated_player_entity = PlayerEntity {
                            action: player_command.action,
                            ..player_entity.clone()
                        };

                        *player_entity = updated_player_entity;
                        // we don't need to store this
                        // tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
                        players_summary.push(player_entity.clone());
                    }
                }
                else {
                    println!("got an unknown player command {}", player_command.action)
                }
            }



            for tile_command in tile_commands_data.iter()
            {
                let region = map.get_region_from_child(tile_command.0);
                let mut tiles = region.lock().await;

                match tiles.get_mut(tile_command.0) {
                    Some(tile) => {
                        let mut updated_tile = tile.clone();
                        // in theory we do something cool here with the tile!!!!
                        match tile_command.1.info {
                            MapCommandInfo::Touch() => {
                                tiles_summary.push(updated_tile);

                                let capacity = tx_me_gameplay_longterm.capacity();
                                server_state.tx_me_gameplay_longterm.store(capacity, std::sync::atomic::Ordering::Relaxed);

                                let capacity = tx_me_gameplay_webservice.capacity();
                                server_state.tx_me_gameplay_webservice.store(capacity, std::sync::atomic::Ordering::Relaxed);

                                // sending the updated tile somewhere.
                                tx_me_gameplay_longterm.send(tile.clone()).await.unwrap();
                                tx_me_gameplay_webservice.send(tile.clone()).await.unwrap();
                            },
                            MapCommandInfo::ChangeHealth(player_id, value) => {
                                println!("Change tile health!!!");
                                let previous_health = tile.health;


                                if previous_health > 0
                                {
                                    updated_tile.health = i32::max(0, updated_tile.health as i32 - value as i32) as u32;
                                    updated_tile.last_update += 1;
                                    tiles_summary.push(updated_tile.clone());
                                    *tile = updated_tile;

                                    let capacity = tx_me_gameplay_longterm.capacity();
                                    server_state.tx_me_gameplay_longterm.store(capacity, std::sync::atomic::Ordering::Relaxed);
                                    let capacity = tx_me_gameplay_webservice.capacity();
                                    server_state.tx_me_gameplay_webservice.store(capacity, std::sync::atomic::Ordering::Relaxed);

                                    // sending the updated tile somewhere.
                                    tx_me_gameplay_longterm.send(tile.clone()).await.unwrap();
                                    tx_me_gameplay_webservice.send(tile.clone()).await.unwrap();


                                    if tile.health == 0
                                    {
                                        let player_option = player_entities.get_mut(&player_id);
                                        if let Some(player_entity) = player_option {
                                            println!("Add inventory item for player");
                                            let new_item = InventoryItem {
                                                item_id: tile.prop,
                                                level: 1,
                                                quality: 1,
                                                amount: 15,
                                            };


                                            player_entity.add_inventory_item(new_item.clone());
                                            // we should also give the player the reward
                                            let reward = PlayerReward {
                                                player_id,
                                                item_id: new_item.item_id,
                                                level: new_item.level,
                                                quality: new_item.quality,
                                                amount: new_item.amount,
                                                inventory_hash : player_entity.inventory_hash
                                            };

                                            players_rewards_summary.push(reward);
                                            tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
                                            players_summary.push(player_entity.clone());
                                        }

                                    }
                                }
                            }
                        }
                    }
                    None => println!("tile not found {}" , tile_command.0),
                }
            }
            // println!("tiles summary {} ", tiles_summary.len());


            tile_commands_data.clear();
            player_commands_data.clear();

            drop(tile_commands_data);
            drop(player_commands_data);

            let tiles_state_update = tiles_summary
                .into_iter()
                .map(|t| StateUpdate::TileState(t));
            let player_presentation_state_update = players_presentation_summary
                .into_iter()
                .map(|p| StateUpdate::PlayerGreetings(p));

            let player_rewards_state_update = players_rewards_summary
                .into_iter()
                .map(|p| StateUpdate::Rewards(p));

            let player_state_updates = players_summary
                .iter()
                .map(|p| StateUpdate::PlayerState(p.clone()));

            let player_attack_state_updates = player_attacks_summary
                .iter()
                .map(|p| StateUpdate::PlayerAttackState(p.clone()));
            // Sending summary to all clients.

            let mut filtered_summary = Vec::new();


    // println!("filtered player state {}", player_state_updates.len());
            filtered_summary.extend(player_state_updates.clone());
            filtered_summary.extend(tiles_state_update.clone());
            filtered_summary.extend(player_presentation_state_update.clone());
            filtered_summary.extend(player_rewards_state_update.clone());
            filtered_summary.extend(player_attack_state_updates.clone());
            let packages = create_data_packets(filtered_summary, &mut packet_number);

            // the data that will be sent to each client is not copied.
            let arc_summary = Arc::new(packages);
            let capacity = tx_bytes_game_socket.capacity();
            server_state.tx_bytes_gameplay_socket.store(capacity, std::sync::atomic::Ordering::Relaxed);
            tx_bytes_game_socket.send(arc_summary).await.unwrap();
        }
    });

    (rx_me_gameplay_longterm, rx_me_gameplay_webservice, rx_pe_gameplay_longterm, tx_mc_webservice_gameplay)
}

pub fn create_data_packets(data : Vec<StateUpdate>, packet_number : &mut u64) -> Vec<Vec<u8>> {
    *packet_number += 1u64;

    let mut buffer = [0u8; 5000];
    let mut start: usize = 1;
    buffer[0] = crate::protocols::Protocol::GlobalState as u8;

    let packet_number_bytes = u64::to_le_bytes(*packet_number); // 8 bytes

    let end: usize = start + 8;
    buffer[start..end].copy_from_slice(&packet_number_bytes);
    start = end;

    let character_presentation_size: usize = 28;
    let player_attack_size: usize = 24;

    let mut stored_bytes:u32 = 0;
    let mut stored_states:u8 = 0;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));


    let mut packets = Vec::<Vec<u8>>::new();
    // this is interesting, this list is shared between threads/clients but since I only read it, it is fine.
    for state_update in data.iter()
    {
        let required_space = match state_update{
            StateUpdate::PlayerState(_) => PLAYER_ENTITY_SIZE as u32 + 1,
            StateUpdate::TileState(_) => MAP_ENTITY_SIZE as u32 + 1,
            StateUpdate::PlayerGreetings(_) => character_presentation_size as u32 + 1,
            StateUpdate::PlayerAttackState(_) => player_attack_size as u32 + 1,
            StateUpdate::Rewards(_) =>player_reward::PLAYER_REWARD_SIZE as u32 +1,
        };

        if stored_bytes + required_space > 5000 // 1 byte for protocol, 8 bytes for the sequence number 
        {
            buffer[start] = DataType::NoData as u8;

            encoder.write_all(buffer.as_slice()).unwrap();
            let compressed_bytes = encoder.reset(Vec::new()).unwrap();
            // println!("compressed {} vs normal {}", compressed_bytes.len(), buffer.len());
            packets.push(compressed_bytes); // this is a copy!

            start = 1;
            stored_states = 0;
            stored_bytes = 0;

            //a new packet with a new sequence number
            *packet_number += 1u64;
            let end: usize = start + 8;
            let packet_number_bytes = u64::to_le_bytes(*packet_number); // 8 bytes
            buffer[start..end].copy_from_slice(&packet_number_bytes);
            start = end;
        }

        match state_update{
            StateUpdate::PlayerState(player_state) => {
                
                buffer[start] = DataType::PlayerState as u8;
                start += 1;

                let player_state_bytes = player_state.to_bytes(); //44
                let next = start + PLAYER_ENTITY_SIZE;
                buffer[start..next].copy_from_slice(&player_state_bytes);
                stored_bytes = stored_bytes + PLAYER_ENTITY_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::TileState(tile_state) => {
                buffer[start] = DataType::TileState as u8;
                start += 1;

                let tile_state_bytes = tile_state.to_bytes();
                let next = start + MapEntity::get_size() as usize;
                buffer[start..next].copy_from_slice(&tile_state_bytes);
                stored_bytes = stored_bytes + MapEntity::get_size() as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            }
            StateUpdate::PlayerGreetings(presentation) => {
                buffer[start] = DataType::PlayerPresentation as u8;
                start += 1;

                let presentation_bytes = presentation.to_bytes(); //28
                let next = start + character_presentation_size;
                buffer[start..next].copy_from_slice(&presentation_bytes);
                stored_bytes = stored_bytes + character_presentation_size as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::PlayerAttackState(player_attack) => {
                buffer[start] = DataType::PlayerAttack as u8;
                start += 1;

                let attack_bytes = player_attack.to_bytes(); //24
                let next = start + player_attack_size;
                buffer[start..next].copy_from_slice(&attack_bytes);
                stored_bytes = stored_bytes + player_attack_size as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::Rewards(player_reward) => {
                buffer[start] = DataType::PlayerReward as u8; // 30
                start += 1;

                let reward_bytes = player_reward.to_bytes(); //16 bytes
                let next = start + player_reward::PLAYER_REWARD_SIZE;
                buffer[start..next].copy_from_slice(&reward_bytes);
                stored_bytes = stored_bytes + player_reward::PLAYER_REWARD_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
        }

    }

    if stored_states > 0
    {
        buffer[start] = DataType::NoData as u8;
        encoder.write_all(&buffer[..(start + 1)]).unwrap();
        // encoder.write_all(buffer.as_slice()).unwrap();
        let compressed_bytes = encoder.reset(Vec::new()).unwrap();
        // println!("compressed {} vs normal {}", compressed_bytes.len(), buffer.len());


        // let data : &[u8] = &compressed_bytes;
        // let mut decoder = ZlibDecoder::new(data);

        // let decoded_data_result :  Result<Vec<u8>, _> = decoder.bytes().collect();
        // let decoded_data = decoded_data_result.unwrap();
        // let decoded_data_array : &[u8] = &decoded_data;

        // println!("data:");
        // println!("{:#04X?}", buffer);

        // println!("decoded data: {}", (buffer == *decoded_data_array));
        packets.push(compressed_bytes); // this is a copy!
    }

    // let all_data : Vec<u8> = packets.iter().flat_map(|d| d.clone()).collect();

    packets
}