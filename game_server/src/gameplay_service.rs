use std::time::SystemTime;
use std::{sync::Arc};

use crate::ServerState;
use crate::character::character_command::{CharacterCommand, self};
use crate::map::GameMap;
use crate::map::map_entity::{MapCommand, MapCommandInfo, MAP_ENTITY_SIZE};
use crate::map::tile_attack::{TileAttack, TILE_ATTACK_SIZE};
use crate::character::character_attack::{CharacterAttack, CHARACTER_ATTACK_SIZE};
use crate::character::character_entity::{InventoryItem, CHARACTER_ENTITY_SIZE, CharacterEntity};
use crate::character::character_reward::{CharacterReward, CHARACTER_REWARD_SIZE, self};
use crate::character::character_presentation::{CharacterPresentation, CHARACTER_PRESENTATION_SIZE};
use crate::map::{tetrahedron_id::TetrahedronId, map_entity::MapEntity};
use crate::real_time_service::client_handler::StateUpdate;
use futures_util::lock::MutexGuard;
use hyper::Server;
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
    TileAttack = 31,
}

pub fn start_service(
    mut rx_pc_client_game : tokio::sync::mpsc::Receiver<CharacterCommand>,
    mut rx_mc_client_game : tokio::sync::mpsc::Receiver<MapCommand>,
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    tx_bytes_game_socket: tokio::sync::mpsc::Sender<Arc<Vec<Vec<u8>>>>
) -> (Receiver<MapEntity>, Receiver<MapEntity>, Receiver<CharacterEntity>, Sender<MapCommand>) {

    let (tx_mc_webservice_gameplay, mut rx_mc_webservice_gameplay ) = tokio::sync::mpsc::channel::<MapCommand>(200);
    let (tx_me_gameplay_longterm, rx_me_gameplay_longterm ) = tokio::sync::mpsc::channel::<MapEntity>(1000);
    let (tx_me_gameplay_webservice, rx_me_gameplay_webservice) = tokio::sync::mpsc::channel::<MapEntity>(1000);
    let (tx_pe_gameplay_longterm, rx_pe_gameplay_longterm ) = tokio::sync::mpsc::channel::<CharacterEntity>(1000);

    //players
    let player_commands = HashMap::<u16,CharacterCommand>::new();
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

    //delayed commands for attacks so they struck a bit later.
    let delayed_tile_commands = Vec::<(u32, MapCommand)>::new();
    let delayed_tile_commands_mutex = Arc::new(Mutex::new(delayed_tile_commands));
    let delayed_tile_commands_lock = delayed_tile_commands_mutex.clone();

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

            let result = std::time::SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
            let current_time = result.ok().map(|d| d.as_secs() as u32);

            let time = &map.time;
            if let Some(new_time) = current_time {
                time.store(new_time, std::sync::atomic::Ordering::Relaxed);
            }
            // println!(" current_time {:?}", current_time);

            let mut players_summary = Vec::new();
            let mut player_attacks_summary = Vec::new();
            let mut tile_attacks_summary = Vec::new();
            let mut players_presentation_summary = Vec::new();
            let mut tiles_summary : Vec<MapEntity>= Vec::new();
            let mut players_rewards_summary : Vec<CharacterReward>= Vec::new();

            // check for delayed_commands
            let mut delayed_commands_lock = delayed_tile_commands_lock.lock().await;

            let mut items_to_execute = Vec::<MapCommand>::new();
            let current_time = time.load(std::sync::atomic::Ordering::Relaxed);

            delayed_commands_lock.retain(|b| {
                let should_execute = b.0 <= current_time;
                println!("checking delayed action {} task_time {} current_time {current_time}", should_execute, b.0);
                if should_execute
                {
                    items_to_execute.push(b.1.clone());
                }

                !should_execute // we keep items that we didn't execute
            });

            drop(delayed_commands_lock);

            let mut player_commands_data = player_commands_processor_lock.lock().await;
            let mut tile_commands_data = tile_commands_processor_lock.lock().await;
            if player_commands_data.len() <= 0  && tile_commands_data.len() <= 0 && items_to_execute.len() <= 0{
                continue;
            }

            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
            for item in player_commands_data.iter()
            {
                let player_command = item.1;
                let cloned_data = item.1.to_owned();

                if let Some(atomic_time) = map.active_players.get(&cloned_data.player_id){
                    atomic_time.store(current_time.as_secs(), std::sync::atomic::Ordering::Relaxed);
                }

                if player_command.action == character_command::IDLE_ACTION {
                    let player_option = player_entities.get_mut(&cloned_data.player_id);
                    if let Some(player_entity) = player_option {
                        let updated_player_entity = CharacterEntity {
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
                else if player_command.action == character_command::GREET_ACTION {
                    let player_option = player_entities.get_mut(&cloned_data.player_id);
                    if let Some(player_entity) = player_option {
                        let name_with_padding = format!("{: <5}", player_entity.character_name);
                        let name_data : Vec<u32> = name_with_padding.chars().into_iter().map(|c| c as u32).collect();
                        let mut name_array = [0u32; 5];
                        name_array.clone_from_slice(&name_data.as_slice()[0..5]);
                        let player_presentation = CharacterPresentation {
                            player_id: player_entity.character_id,
                            character_name: name_array,
                        };

                        players_presentation_summary.push(player_presentation);
                    }

                }
                else if player_command.action == character_command::RESPAWN_ACTION { // respawn, we only update health for the moment
                    let player_option = player_entities.get_mut(&cloned_data.player_id);
                    if let Some(player_entity) = player_option {
                        let updated_player_entity = CharacterEntity {
                            action: player_command.action,
                            health: player_entity.constitution,
                            ..player_entity.clone()
                        };

                        *player_entity = updated_player_entity;
                        tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
                        players_summary.push(player_entity.clone());
                    }
                }
                else if player_command.action == character_command::WALK_ACTION { // respawn, we only update health for the moment
                    let player_option = player_entities.get_mut(&cloned_data.player_id);
                    if let Some(player_entity) = player_option {
                        let updated_player_entity = CharacterEntity {
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
                else if player_command.action == character_command::ATTACK_ACTION { 

                    // we anounce the attack
                    let attack = CharacterAttack{
                        player_id: cloned_data.player_id,
                        target_player_id: cloned_data.other_player_id,
                        damage: 2,
                        skill_id: 0,
                        target_tile_id: TetrahedronId::from_string("a0"), // we need a default value
                    };
                    player_attacks_summary.push(attack);


                    if player_command.required_time > 1 {

                    }

                    let player_option = player_entities.get_mut(&cloned_data.player_id);
                    if let Some(player_entity) = player_option {
                        let updated_player_entity = CharacterEntity {
                            action: player_command.action,
                            ..player_entity.clone()
                        };
                        let player_attack = updated_player_entity.attack;
                        *player_entity = updated_player_entity;
                        tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
                        players_summary.push(player_entity.clone());

                        // solo hay que aplicar el danio 
                        if let Some(other_entity) = player_entities.get_mut(&cloned_data.other_player_id){
                            let result = other_entity.health.saturating_sub(player_attack);
                            let updated_player_entity = CharacterEntity {
                                action: other_entity.action,
                                health: result,
                                ..other_entity.clone()
                            };

                            *other_entity = updated_player_entity;
                            tx_pe_gameplay_longterm.send(other_entity.clone()).await.unwrap();
                            players_summary.push(other_entity.clone());
                        }
                    }
                }
                else if player_command.action == character_command::ATTACK_TILE_ACTION
                || player_command.action == character_command::BUILD_ACTION { // respawn, we only update health for the moment
                    let player_option = player_entities.get_mut(&cloned_data.player_id);
                    if let Some(player_entity) = player_option {
                        let updated_player_entity = CharacterEntity {
                            action: player_command.action,
                            ..player_entity.clone()
                        };

                        *player_entity = updated_player_entity;
                        // we don't need to store this
                        // tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
                        players_summary.push(player_entity.clone());
                    }
                }
                // else {
                //     println!("got an unknown player command {}", player_command.action)
                // }
            }

            drop(player_entities);

            for tile_command in items_to_execute.iter()
            {
                let region = map.get_region_from_child(&tile_command.id);
                let mut tiles = region.lock().await;

                match &tile_command.info {
                    MapCommandInfo::Touch() => todo!(),
                    MapCommandInfo::ChangeHealth(_, _) => todo!(),
                    MapCommandInfo::LayFoundation(_, _, _, _, _) => todo!(),
                    MapCommandInfo::BuildStructure(_, _) => todo!(),
                    MapCommandInfo::AttackWalker(player_id, _required_time) => {
                        let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
                        let player_option = player_entities.get_mut(&player_id);
                        if let Some(player_entity) = player_option {
                            if player_entity.health > 0  
                                // && updated_tile.faction != 0 
                                // && updated_tile.faction != player_entity.faction 
                            {
                                let result = player_entity.health.saturating_sub(2);
                                let updated_player_entity = CharacterEntity {
                                    action: player_entity.action,
                                    health: result,
                                    ..player_entity.clone()
                                };

                                *player_entity = updated_player_entity.clone();
                                drop(player_entities);
                                tx_pe_gameplay_longterm.send(updated_player_entity.clone()).await.unwrap();
                                players_summary.push(updated_player_entity.clone());
                            }
                        }
                    },
                    MapCommandInfo::SpawnMob(_) => todo!(),
                    MapCommandInfo::MoveMob(_, _, _, _) => todo!(),
                    MapCommandInfo::ControlMob(_, _) => todo!(),
                    MapCommandInfo::AttackMob(player_id, damage, _required_time) => {

                        if let Some(tile) = tiles.get_mut(&tile_command.id) {
                            let (updated_tile, reward) = process_tile_attack(
                                damage, 
                                tile, 
                            );
                            
                            *tile = updated_tile.clone();
                            drop(tiles);

                            report_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                            // sending the updated tile somewhere.
                            tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                            tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                            tiles_summary.push(updated_tile.clone());

                            if let Some(reward) = reward {
                                let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
                                let player_option = player_entities.get_mut(&player_id);
                                if let Some(player_entity) = player_option {
                                    update_character_entity(player_entity,reward,&mut players_rewards_summary, &mut players_summary);
                                    let updated_player_entity = player_entity.clone();
                                    drop(player_entities);
                                    // we try to drop any locks before doing an await
                                    tx_pe_gameplay_longterm.send(updated_player_entity.clone()).await.unwrap();
                                }
                            }
                            println!("process tile attack ended");
                        } // end of if let
                    } // end of map command map
                }
            }

            // drop(delayed_commands_lock);

            for tile_command in tile_commands_data.iter()
            {
                let region = map.get_region_from_child(tile_command.0);
                let mut tiles = region.lock().await;

                match tiles.get_mut(tile_command.0) {
                    Some(tile) => {
                        let mut updated_tile = tile.clone();
                        // in theory we do something cool here with the tile!!!!
                        match &tile_command.1.info {
                            MapCommandInfo::Touch() => {
                                tiles_summary.push(updated_tile.clone());

                                drop(tiles);
                                report_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                                // sending the updated tile somewhere.
                                tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                                tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                            },
                            MapCommandInfo::ChangeHealth(player_id, damage) => {
                                println!("Change tile health!!!");
                                let previous_health = tile.health;

                                // this means this tile is being built
                                if tile.health > tile.constitution 
                                {
                                    updated_tile.constitution = i32::max(0, updated_tile.constitution as i32 - *damage as i32) as u32;
                                    updated_tile.version += 1;
                                    if updated_tile.constitution == 0
                                    {
                                        updated_tile.prop = 0;
                                        updated_tile.health = 0;
                                    }

                                    tiles_summary.push(updated_tile.clone());
                                    *tile = updated_tile.clone();
                                    drop(tiles);

                                    report_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                                    // sending the updated tile somewhere.
                                    tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                                    tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                                }
                                else if previous_health > 0
                                {
                                    let collected_prop = updated_tile.prop;
                                    updated_tile.health = i32::max(0, updated_tile.health as i32 - *damage as i32) as u32;
                                    updated_tile.version += 1;
                                    if updated_tile.health == 0
                                    {
                                        updated_tile.prop = 0;
                                    }
                                    tiles_summary.push(updated_tile.clone());
                                    *tile = updated_tile.clone();
                                    drop(tiles);

                                    report_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                                    // sending the updated tile somewhere.
                                    tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                                    tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();


                                    if updated_tile.health == 0
                                    {
                                        let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
                                        let player_option = player_entities.get_mut(&player_id);
                                        if let Some(player_entity) = player_option {
                                            println!("Add inventory item for player");
                                            let new_item = InventoryItem {
                                                item_id: collected_prop + 2, // this is to use 0 and 1 as soft and hard currency, we need to read definitions...
                                                level: 1,
                                                quality: 1,
                                                amount: 1,
                                            };


                                            player_entity.add_inventory_item(new_item.clone());

                                            let updated_player_entity = player_entity.clone();

                                            drop(player_entities);
                                            // we should also give the player the reward
                                            let reward = CharacterReward {
                                                player_id: *player_id,
                                                item_id: new_item.item_id,
                                                level: new_item.level,
                                                quality: new_item.quality,
                                                amount: new_item.amount,
                                                inventory_hash : updated_player_entity.inventory_hash
                                            };

                                            println!("reward {:?}", reward);

                                            players_rewards_summary.push(reward);
                                            tx_pe_gameplay_longterm.send(updated_player_entity.clone()).await.unwrap();
                                            players_summary.push(updated_player_entity.clone());
                                        }
                                    }
                                }
                                else
                                {
                                    tiles_summary.push(updated_tile.clone());
                                }
                            }, // we need to deduct stuff from the player
                            MapCommandInfo::LayFoundation(player_id, prop, pathness_a, pathness_b,pathness_c) => {

                                if updated_tile.prop == 0
                                {
                                    updated_tile.health = 500;
                                    updated_tile.constitution = 0;
                                    updated_tile.prop = *prop;

                                    let player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;

                                    let player_option = player_entities.get(&player_id);
                                    if let Some(player_entity) = player_option {
                                        updated_tile.faction = player_entity.faction;
                                    }

                                    drop(player_entities);

                                    updated_tile.version += 1;
                                    tiles_summary.push(updated_tile.clone());
                                    *tile = updated_tile.clone();
                                    drop(tiles);

                                    report_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                                    // sending the updated tile somewhere.
                                    tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                                    tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                                }
                                else {
                                    tiles_summary.push(updated_tile.clone());
                                }
                            },
                            MapCommandInfo::BuildStructure(_player_id, increment) => {
                                
                                if updated_tile.health > updated_tile.constitution {

                                    updated_tile.constitution = i32::min(updated_tile.health as i32, updated_tile.constitution as i32 + *increment as i32) as u32;
                                    updated_tile.version += 1;
                                    tiles_summary.push(updated_tile.clone());
                                    *tile = updated_tile.clone();
                                    drop(tiles);

                                    report_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                                    // sending the updated tile somewhere.
                                    tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                                    tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                                }
                                else {
                                    // we send the tile in case the one thinking that the structure is not built yet will receive the tile
                                    tiles_summary.push(updated_tile.clone());
                                    println!("structure is already built!");
                                    // structure is already built!
                                }
                            },
                            MapCommandInfo::AttackWalker(player_id, required_time) => {
                                if *required_time > 0 {
                                    let mut lock = delayed_tile_commands_lock.lock().await;

                                    let current_time = time.load(std::sync::atomic::Ordering::Relaxed);
                                    let info = MapCommandInfo::AttackWalker(*player_id, *required_time);
                                    let map_action = MapCommand { id: tile.id.clone(), info };
                                    lock.push((current_time + *required_time as u32, map_action));

                                    drop(lock);
                                }
                                else {
                                    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
                                    let player_option = player_entities.get_mut(&player_id);
                                    if let Some(player_entity) = player_option {
                                        if player_entity.health > 0  
                                            // && updated_tile.faction != 0 
                                            // && updated_tile.faction != player_entity.faction 
                                        {
                                            let result = player_entity.health.saturating_sub(2);
                                            let updated_player_entity = CharacterEntity {
                                                action: player_entity.action,
                                                health: result,
                                                ..player_entity.clone()
                                            };

                                            *player_entity = updated_player_entity.clone();
                                            drop(player_entities);
                                            tx_pe_gameplay_longterm.send(updated_player_entity.clone()).await.unwrap();
                                            players_summary.push(updated_player_entity.clone());

                                        }
                                    }
                                }


                                // updating tile stuff inmediately.
                                updated_tile.version += 1;

                                if updated_tile.owner_id == *player_id {
                                    // the controller is fighting this mob, we give him more control
                                    let current_time = time.load(std::sync::atomic::Ordering::Relaxed);
                                    updated_tile.ownership_time = current_time + 5; // more seconds of control
                                }
                                *tile = updated_tile.clone();
                                drop(tiles);


                                let attack = TileAttack{
                                    tile_id: updated_tile.id.clone(),
                                    target_player_id: *player_id,
                                    damage: 2,
                                    skill_id: 0,
                                };
                                tile_attacks_summary.push(attack);
                            },
                            MapCommandInfo::SpawnMob(mob_id) => {

                                if updated_tile.prop == 0 // we can spawn a mob here.
                                {
                                    updated_tile.health = 100;
                                    updated_tile.constitution = 100;
                                    updated_tile.prop = *mob_id;
                                    updated_tile.origin_id = tile.id.clone();
                                    updated_tile.target_id = tile.id.clone();
                                    updated_tile.faction = 4;// corruption faction

                                    updated_tile.version += 1;
                                    tiles_summary.push(updated_tile.clone());
                                    *tile = updated_tile.clone();
                                    drop(tiles);

                                    report_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                                    // sending the updated tile somewhere.
                                    tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                                    tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                                }
                                else {
                                    tiles_summary.push(updated_tile.clone());
                                }
                            },
                            MapCommandInfo::MoveMob(player_id, mob_id, new_tile_id, distance) => {

                                let current_time = time.load(std::sync::atomic::Ordering::Relaxed);
                                // we also need to be sure this player has control over the tile
                                if updated_tile.prop == *mob_id // we are mostly sure you know this is a mob and wants to move 
                                    && &updated_tile.target_id != new_tile_id
                                    && updated_tile.time < current_time // only if you are not doing something already
                                    && updated_tile.owner_id == *player_id
                                {
                                    updated_tile.version += 1;
                                    let required_time = u32::max(1, (*distance / 0.5f32).ceil() as u32);
                                    updated_tile.time = current_time + required_time;
                                    updated_tile.origin_id = tile.target_id.clone();
                                    updated_tile.target_id = new_tile_id.clone();

                                    updated_tile.ownership_time = current_time + 10; // more seconds of control
                                    // println!("updating ownership time {}" , updated_tile.ownership_time);

                                    tiles_summary.push(updated_tile.clone());
                                    *tile = updated_tile.clone();
                                    drop(tiles);

                                    report_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                                    // sending the updated tile somewhere.
                                    tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                                    tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                                }
                                else {
                                    tiles_summary.push(updated_tile.clone());
                                }
                            },
                            MapCommandInfo::ControlMob(player_id, mob_id) => {
                                let current_time = time.load(std::sync::atomic::Ordering::Relaxed);
                                if updated_tile.prop == *mob_id // we are mostly sure you know this is a mob and wants to move 
                                    && updated_tile.ownership_time < current_time // owner timeout
                                {
                                    // println!("updating time {current_time} {}", updated_tile.ownership_time);
                                    updated_tile.version += 1;
                                    updated_tile.owner_id = *player_id;
                                    updated_tile.ownership_time = current_time + 10; // seconds of control
                                    // println!("new time {}", updated_tile.ownership_time);

                                    tiles_summary.push(updated_tile.clone());
                                    *tile = updated_tile.clone();
                                    drop(tiles);

                                    report_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                                    // sending the updated tile somewhere.
                                    tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                                    tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                                }
                                else {
                                    // println!("Somethign failed {current_time} {}", updated_tile.ownership_time);
                                    tiles_summary.push(updated_tile.clone());
                                }
                            },
                            // this is very similar to change health command, but here we need to send and arrow.
                            MapCommandInfo::AttackMob(player_id, damage, required_time) => {
                                let tile_id = tile.id.clone();
                                println!("required time for attack {required_time}");
                                if *required_time > 0 {
                                    let mut lock = delayed_tile_commands_lock.lock().await;

                                    let current_time = time.load(std::sync::atomic::Ordering::Relaxed);
                                    let info = MapCommandInfo::AttackMob(*player_id, *damage, *required_time);
                                    let map_action = MapCommand { id: tile.id.clone(), info };
                                    lock.push((current_time + *required_time as u32, map_action));

                                    drop(lock);
                                }
                                else {
                                    // this code is repeated
                                    // if let Some(tile) = tiles.get_mut(&tile_command.id)
                                    {
                                        let (updated_tile, reward) = process_tile_attack(
                                            damage, 
                                            tile, 
                                        );
                                        
                                        *tile = updated_tile.clone();
                                        drop(tiles);

                                        report_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                                        // sending the updated tile somewhere.
                                        tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                                        tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                                        tiles_summary.push(updated_tile.clone());

                                        if let Some(reward) = reward {
                                            let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
                                            let player_option = player_entities.get_mut(&player_id);
                                            if let Some(player_entity) = player_option {
                                                update_character_entity(player_entity,reward,&mut players_rewards_summary, &mut players_summary);
                                                let updated_player_entity = player_entity.clone();
                                                drop(player_entities);
                                                // we try to drop any locks before doing an await
                                                tx_pe_gameplay_longterm.send(updated_player_entity.clone()).await.unwrap();
                                            }
                                        }
                                        // println!("process tile attack ended");
                                    } // end of if let
                                }

                                let attack = CharacterAttack{
                                    player_id: *player_id,
                                    target_player_id: 0,
                                    damage: 2,
                                    skill_id: 0,
                                    target_tile_id: tile_id.clone(),
                                };
                                player_attacks_summary.push(attack);
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

            let tile_attack_state_updates = tile_attacks_summary
                .iter()
                .map(|p| StateUpdate::TileAttackState(p.clone()));
            // Sending summary to all clients.

            let mut filtered_summary = Vec::new();


    // println!("filtered player state {}", player_state_updates.len());
            filtered_summary.extend(player_state_updates.clone());
            filtered_summary.extend(tiles_state_update.clone());
            filtered_summary.extend(player_presentation_state_update.clone());
            filtered_summary.extend(player_rewards_state_update.clone());
            filtered_summary.extend(player_attack_state_updates.clone());
            filtered_summary.extend(tile_attack_state_updates.clone());
            // println!("filtered summarny total {}" , filtered_summary.len());
            if filtered_summary.len() > 0 
            {
                let packages = create_data_packets(filtered_summary, &mut packet_number);
                // the data that will be sent to each client is not copied.
                let arc_summary = Arc::new(packages);
                let capacity = tx_bytes_game_socket.capacity();
                server_state.tx_bytes_gameplay_socket.store(capacity, std::sync::atomic::Ordering::Relaxed);
                tx_bytes_game_socket.send(arc_summary).await.unwrap();
            }
        }
    });

    (rx_me_gameplay_longterm, rx_me_gameplay_webservice, rx_pe_gameplay_longterm, tx_mc_webservice_gameplay)
}

pub fn update_character_entity(
    player_entity : &mut CharacterEntity, 
    reward : InventoryItem,
    players_rewards_summary : &mut Vec<CharacterReward>,
    players_summary : &mut Vec<CharacterEntity>)
{
        player_entity.add_inventory_item(reward.clone());
        // we should also give the player the reward
        let reward = CharacterReward {
            player_id: player_entity.character_id,
            item_id: reward.item_id,
            level: reward.level,
            quality: reward.quality,
            amount: reward.amount,
            inventory_hash : player_entity.inventory_hash
        };

        println!("reward {:?}", reward);

        players_rewards_summary.push(reward);
        players_summary.push(player_entity.clone());
}

pub fn report_capacity(
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    server_state : Arc<ServerState>
){
    let capacity = tx_me_gameplay_longterm.capacity();
    server_state.tx_me_gameplay_longterm.store(capacity, std::sync::atomic::Ordering::Relaxed);
    let capacity = tx_me_gameplay_webservice.capacity();
    server_state.tx_me_gameplay_webservice.store(capacity, std::sync::atomic::Ordering::Relaxed);
}

pub fn process_tile_attack(
    damage: &u16, 
    tile : &mut MapEntity, 
) -> (MapEntity, Option<InventoryItem>)
{
    // let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
    let mut updated_tile : MapEntity = tile.clone();
    let mut reward : Option<InventoryItem> = None;
    println!("Change mob health!!!");
    let previous_health = tile.health;

    // this means this tile is being built
    if tile.health > tile.constitution 
    {
        updated_tile.constitution = i32::max(0, updated_tile.constitution as i32 - *damage as i32) as u32;
        updated_tile.version += 1;
        if updated_tile.constitution == 0
        {
            updated_tile.prop = 0;
            updated_tile.health = 0;
        }
    }
    else if previous_health > 0
    {
        let collected_prop = updated_tile.prop;
        updated_tile.health = i32::max(0, updated_tile.health as i32 - *damage as i32) as u32;
        updated_tile.version += 1;
        if updated_tile.health == 0
        {
            updated_tile.prop = 0;
        }

        if tile.health == 0
        {
            println!("Add inventory item for player");

            reward = Some(InventoryItem {
                item_id: collected_prop + 2, // this is to use 0 and 1 as soft and hard currency, we need to read definitions...
                level: 1,
                quality: 1,
                amount: 1,
            });
        }
    }
    (updated_tile, reward)
}

pub fn create_data_packets(data : Vec<StateUpdate>, packet_number : &mut u64) -> Vec<Vec<u8>> {
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


    let mut packets = Vec::<Vec<u8>>::new();
    // this is interesting, this list is shared between threads/clients but since I only read it, it is fine.

    // println!("data to send {}" , data.len());
    for state_update in data.iter()
    {
        let required_space = match state_update{
            StateUpdate::PlayerState(_) => CHARACTER_ENTITY_SIZE as u32 + 1,
            StateUpdate::TileState(_) => MAP_ENTITY_SIZE as u32 + 1,
            StateUpdate::PlayerGreetings(_) => CHARACTER_PRESENTATION_SIZE as u32 + 1,
            StateUpdate::PlayerAttackState(_) => CHARACTER_ATTACK_SIZE as u32 + 1,
            StateUpdate::Rewards(_) =>CHARACTER_REWARD_SIZE as u32 +1,
            StateUpdate::TileAttackState(_) =>TILE_ATTACK_SIZE as u32 +1,
        };

        // let sent_data = match state_update{
        //     StateUpdate::PlayerState(_) => "player state".to_owned(),
        //     StateUpdate::TileState(_) => "tile state".to_owned(),
        //     StateUpdate::PlayerGreetings(_) => "presentation".to_owned(),
        //     StateUpdate::PlayerAttackState(_) => "player attack state ". to_owned(),
        //     StateUpdate::Rewards(_) => "player requred".to_owned(),
        //     StateUpdate::TileAttackState(_) =>"tile attack state".to_owned(),
        // };

        // println!("data sent {} required space {}",sent_data, required_space);



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
            StateUpdate::PlayerState(player_state) => {
                
                buffer[start] = DataType::PlayerState as u8;
                start += 1;

                let player_state_bytes = player_state.to_bytes(); //44
                let next = start + CHARACTER_ENTITY_SIZE;
                buffer[start..next].copy_from_slice(&player_state_bytes);
                stored_bytes = stored_bytes + CHARACTER_ENTITY_SIZE as u32 + 1;
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
                let next = start + CHARACTER_PRESENTATION_SIZE;
                buffer[start..next].copy_from_slice(&presentation_bytes);
                stored_bytes = stored_bytes + CHARACTER_PRESENTATION_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::PlayerAttackState(player_attack) => {
                buffer[start] = DataType::PlayerAttack as u8;
                start += 1;

                let attack_bytes = player_attack.to_bytes(); //24
                let next = start + CHARACTER_ATTACK_SIZE;
                buffer[start..next].copy_from_slice(&attack_bytes);
                stored_bytes = stored_bytes + CHARACTER_ATTACK_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::Rewards(player_reward) => {
                buffer[start] = DataType::PlayerReward as u8; // 30
                start += 1;

                let reward_bytes = player_reward.to_bytes(); //16 bytes
                let next = start + character_reward::CHARACTER_REWARD_SIZE;
                buffer[start..next].copy_from_slice(&reward_bytes);
                stored_bytes = stored_bytes + character_reward::CHARACTER_REWARD_SIZE as u32 + 1;
                stored_states = stored_states + 1;
                start = next;
            },
            StateUpdate::TileAttackState(tile_attack) => {
                buffer[start] = DataType::TileAttack as u8;
                start += 1;

                let attack_bytes = tile_attack.to_bytes(); //22
                let next = start + TILE_ATTACK_SIZE;
                buffer[start..next].copy_from_slice(&attack_bytes);
                stored_bytes = stored_bytes + TILE_ATTACK_SIZE as u32 + 1;
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
        packets.push(compressed_bytes); // this is a copy!
    }

    // let all_data : Vec<u8> = packets.iter().flat_map(|d| d.clone()).collect();

    packets
}