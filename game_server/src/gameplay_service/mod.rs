use std::time::SystemTime;
use std::sync::Arc;

use crate::ServerState;
use crate::character::character_command::{CharacterCommand, self};
use crate::gameplay_service::utils::update_character_entity;
use crate::map::GameMap;
use crate::map::map_entity::{MapCommand, MapCommandInfo};
use crate::character::character_attack::CharacterAttack;
use crate::character::character_entity::{InventoryItem, CharacterEntity};
use crate::character::character_reward::CharacterReward;
use crate::character::character_presentation::CharacterPresentation;
use crate::map::tile_attack::TileAttack;
use crate::map::{tetrahedron_id::TetrahedronId, map_entity::MapEntity};
use crate::real_time_service::client_handler::StateUpdate;
use crate::tower::TowerCommand;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use std::collections::HashMap;

use self::utils::{report_capacity, process_tile_attack, get_tile_commands_to_execute, get_player_commands_to_execute};
pub mod data_packer;
pub mod utils;

pub enum DataType
{
    NoData = 25,
    PlayerState = 26,
    TileState = 27,
    PlayerPresentation = 28,
    PlayerAttack = 29,
    PlayerReward = 30,
    TileAttack = 31,
    TowerState = 32,
}

pub fn start_service(
    mut rx_pc_client_game : tokio::sync::mpsc::Receiver<CharacterCommand>,
    mut rx_mc_client_game : tokio::sync::mpsc::Receiver<MapCommand>,
    mut rx_tc_client_game : tokio::sync::mpsc::Receiver<TowerCommand>,
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    tx_bytes_game_socket: tokio::sync::mpsc::Sender<Vec<(u64, Vec<u8>)>>
) -> (Receiver<MapEntity>, Receiver<MapEntity>, Receiver<CharacterEntity>, Sender<MapCommand>) {

    let (tx_mc_webservice_gameplay, mut rx_mc_webservice_gameplay ) = tokio::sync::mpsc::channel::<MapCommand>(200);
    let (tx_me_gameplay_longterm, rx_me_gameplay_longterm ) = tokio::sync::mpsc::channel::<MapEntity>(1000);
    let (tx_me_gameplay_webservice, rx_me_gameplay_webservice) = tokio::sync::mpsc::channel::<MapEntity>(1000);
    let (tx_pe_gameplay_longterm, rx_pe_gameplay_longterm ) = tokio::sync::mpsc::channel::<CharacterEntity>(1000);

    //players
    //player commands -------------------------------------
    let player_commands = Vec::<CharacterCommand>::new();
    let player_commands_mutex = Arc::new(Mutex::new(player_commands));
    let player_commands_processor_lock = player_commands_mutex.clone();
    let player_commands_agregator_lock = player_commands_mutex.clone();

    //tile commands -------------------------------------
    let tile_commands = Vec::<MapCommand>::new();
    let tile_commands_mutex = Arc::new(Mutex::new(tile_commands));
    let tile_commands_processor_lock = tile_commands_mutex.clone();

    let tile_commands_agregator_from_client_lock = tile_commands_mutex.clone();
    let tile_commands_agregator_from_webservice_lock = tile_commands_mutex.clone();

    //tower commands -------------------------------------
    let tower_commands = Vec::<TowerCommand>::new();
    let tower_commands_mutex = Arc::new(Mutex::new(tower_commands));
    let tower_commands_processor_lock = tower_commands_mutex.clone();

    let tower_commands_agregator_from_client_lock = tower_commands_mutex.clone();
    let tower_commands_agregator_from_webservice_lock = tower_commands_mutex.clone();

    //delayed commands for attacks so they struck a bit later.
    let delayed_tile_commands = Vec::<(u64, MapCommand)>::new();
    let delayed_tile_commands_mutex = Arc::new(Mutex::new(delayed_tile_commands));
    let delayed_tile_commands_lock = delayed_tile_commands_mutex.clone();

    //delayed commands for attacks so they struck a bit later.
    let delayed_player_commands = Vec::<(u64, u16)>::new();
    let delayed_player_commands_mutex = Arc::new(Mutex::new(delayed_player_commands));
    let delayed_player_commands_lock = delayed_player_commands_mutex.clone();


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
            
            data.push(message);
        }
    });

    // task that gathers world changes comming from a client into a list.
    tokio::spawn(async move {
        // let mut sequence_number:u64 = 101;
        loop {
            let message = rx_mc_client_game.recv().await.unwrap();
            // println!("got a tile change data {}", message.id);
            let mut data = tile_commands_agregator_from_client_lock.lock().await;
            data.push(message);
        }
    });

    tokio::spawn(async move {

        // let mut sequence_number:u64 = 101;
        loop {
            let message = rx_tc_client_game.recv().await.unwrap();
            // println!("got a tile change data {}", message.id);
            let mut data = tower_commands_agregator_from_client_lock.lock().await;
            data.push(message);
        }
    });

    // task that gathers world changes comming from web service into a list.
    tokio::spawn(async move {
        // let mut sequence_number:u64 = 101;
        loop {
            let message = rx_mc_webservice_gameplay.recv().await.unwrap();
            // println!("got a tile change data {}", message.id);
            let mut data = tile_commands_agregator_from_webservice_lock.lock().await;
            data.push(message);
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
            let current_time = result.ok().map(|d| d.as_millis() as u64);

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

            let current_time = time.load(std::sync::atomic::Ordering::Relaxed);

            // check for delayed_commands
            // check if tile commands can be executed.
            let mut delayed_tile_commands_guard = delayed_tile_commands_lock.lock().await;
            let delayed_tile_commands_to_execute = get_tile_commands_to_execute(current_time, &mut delayed_tile_commands_guard);
            drop(delayed_tile_commands_guard);

            // check for delayed_commands from player
            let mut delayed_player_commands_guards = delayed_player_commands_lock.lock().await;
            let delayed_player_commands_to_execute = get_player_commands_to_execute(current_time, &mut delayed_player_commands_guards);
            drop(delayed_player_commands_guards);

            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();

            let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
            let mut player_commands_data = player_commands_processor_lock.lock().await;
            if player_commands_data.len() > 0  
            {
                for player_command in player_commands_data.iter()
                {
                    let cloned_data = player_command.to_owned();

                    if let Some(atomic_time) = map.active_players.get(&cloned_data.player_id){
                        atomic_time.store(current_time.as_secs(), std::sync::atomic::Ordering::Relaxed);
                    }

                    if player_command.action == character_command::IDLE_ACTION {
                        let player_option = player_entities.get_mut(&cloned_data.player_id);
                        if let Some(player_entity) = player_option {
                            let updated_player_entity = CharacterEntity {
                                action: player_command.action,
                                version: player_entity.version + 1,
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
                                version: player_entity.version + 1,
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
                                version: player_entity.version + 1,
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

                        let player_option = player_entities.get_mut(&cloned_data.player_id);
                        if let Some(player_entity) = player_option {
                            let updated_player_entity = CharacterEntity {
                                action: player_command.action,
                                version: player_entity.version + 1,
                                ..player_entity.clone()
                            };
                            *player_entity = updated_player_entity;
                            tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
                            players_summary.push(player_entity.clone());
                        }

                        // if player_command.required_time > 1 {
                        let mut lock = delayed_player_commands_lock.lock().await;
                        let current_time = time.load(std::sync::atomic::Ordering::Relaxed);
                        // println!("push attack in required time {}", player_command.required_time);
                        lock.push((current_time + player_command.required_time as u64, player_command.other_player_id));
                        drop(lock);
                    }
                    else if player_command.action == character_command::ATTACK_TILE_ACTION
                    || player_command.action == character_command::BUILD_ACTION { // respawn, we only update health for the moment
                        let player_option = player_entities.get_mut(&cloned_data.player_id);
                        if let Some(player_entity) = player_option {
                            let updated_player_entity = CharacterEntity {
                                action: player_command.action,
                                version: player_entity.version + 1,
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
                player_commands_data.clear();
                drop(player_commands_data);
            }

            // println!("delayed player commands to execute {}" , player_commands_to_execute.len()); 
            for player_command in delayed_player_commands_to_execute.iter()
            {
                if let Some(other_entity) = player_entities.get_mut(player_command)
                {
                    let result = other_entity.health.saturating_sub(11);
                    let updated_player_entity = CharacterEntity {
                        action: other_entity.action,
                        version: other_entity.version + 1,
                        health: result,
                        ..other_entity.clone()
                    };

                    *other_entity = updated_player_entity;
                    tx_pe_gameplay_longterm.send(other_entity.clone()).await.unwrap();
                    players_summary.push(other_entity.clone());
                }
            }

            drop(player_entities);

            for tile_command in delayed_tile_commands_to_execute.iter()
            {
                let region = map.get_region_from_child(&tile_command.id);
                let mut tiles = region.lock().await;

                match &tile_command.info {
                    MapCommandInfo::Touch() => todo!(),
                    MapCommandInfo::ChangeHealth(_, _) => todo!(),
                    MapCommandInfo::LayFoundation(_, _, _, _, _) => todo!(),
                    MapCommandInfo::BuildStructure(_, _) => todo!(),
                    MapCommandInfo::AttackWalker(player_id, _required_time) => {
                        drop(tiles);
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
                                    version: player_entity.version + 1,
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
                    MapCommandInfo::MoveMob(_, _, _, _, _) => todo!(),
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
                                println!("We got some reward {:?}", reward);
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
                        } // end of if let
                    } // end of map command map
                }
            }

            // drop(delayed_commands_lock);

            let mut tile_commands_data = tile_commands_processor_lock.lock().await;
            if tile_commands_data.len() > 0 
            {
                for tile_command in tile_commands_data.iter()
                {
                    let region = map.get_region_from_child(&tile_command.id);
                    let mut tiles = region.lock().await;

                    match tiles.get_mut(&tile_command.id) {
                        Some(tile) => {
                            let mut updated_tile = tile.clone();
                            // in theory we do something cool here with the tile!!!!
                            match &tile_command.info {
                                MapCommandInfo::Touch() => {
                                    tiles_summary.push(updated_tile.clone());

                                    drop(tiles);
                                    report_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                                    // sending the updated tile somewhere.
                                    tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                                    tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                                },
                                MapCommandInfo::ChangeHealth(player_id, damage) => {
                                    println!("Change tile health!!! {}", tile.prop);
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
                                            println!("updated tile is now 0");
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
                                                player_entity.version += 1;

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
                                MapCommandInfo::LayFoundation(player_id, prop, _pathness_a, _pathness_b,_pathness_c) => {

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

                                    // updating tile stuff inmediately and releasing lock before another await.
                                    updated_tile.version += 1;

                                    if updated_tile.owner_id == *player_id {
                                        // the controller is fighting this mob, we give him more control
                                        let current_time = time.load(std::sync::atomic::Ordering::Relaxed);
                                        updated_tile.ownership_time = (current_time / 1000) as u32 + 5; // more seconds of control
                                    }
                                    *tile = updated_tile.clone();
                                    let tile_id = tile.id.clone();
                                    drop(tiles);

                                    let attack = TileAttack{
                                        tile_id: updated_tile.id.clone(),
                                        target_player_id: *player_id,
                                        damage: 2,
                                        skill_id: 0,
                                    };
                                    tile_attacks_summary.push(attack);

                                    // now we push the delayed message.

                                    let mut lock = delayed_tile_commands_lock.lock().await;
                                    let current_time = time.load(std::sync::atomic::Ordering::Relaxed);
                                    let info = MapCommandInfo::AttackWalker(*player_id, *required_time);

                                    let map_action = MapCommand { id: tile_id, info };
                                    lock.push((current_time + *required_time as u64, map_action));
                                    drop(lock);

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
                                MapCommandInfo::MoveMob(player_id, mob_id, new_tile_id, _distance, required_time) => {

                                    let current_time = time.load(std::sync::atomic::Ordering::Relaxed) / 1000;
                                    let current_time = current_time as u32;
                                    // we also need to be sure this player has control over the tile
                                    if updated_tile.prop == *mob_id // we are mostly sure you know this is a mob and wants to move 
                                        && &updated_tile.target_id != new_tile_id
                                        && updated_tile.time < current_time // only if you are not doing something already
                                        && updated_tile.owner_id == *player_id
                                    {
                                        updated_tile.version += 1;
                                        // let required_time = u32::max(1, (*distance / 0.5f32).ceil() as u32);
                                        let required_time = required_time.round() as u32;
                                        // println!("required time {} " , required_time);
                                        updated_tile.time = (current_time / 1000) + required_time;
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
                                    let current_time = time.load(std::sync::atomic::Ordering::Relaxed) / 1000;
                                    let current_time = current_time as u32;
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
                                    // println!("required time for attack {required_time}");

                                    let mut lock = delayed_tile_commands_lock.lock().await;
                                    let current_time = time.load(std::sync::atomic::Ordering::Relaxed);
                                    let info = MapCommandInfo::AttackMob(*player_id, *damage, *required_time);
                                    let map_action = MapCommand { id: tile.id.clone(), info };
                                    lock.push((current_time + *required_time as u64, map_action));

                                    drop(lock);

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
                        None => println!("tile not found {}" , tile_command.id),
                    }
                }
            }
            
            // println!("tiles summary {} ", tiles_summary.len());
            tile_commands_data.clear();
            drop(tile_commands_data);

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
                let packages = data_packer::create_data_packets(filtered_summary, &mut packet_number);
                // the data that will be sent to each client is not copied.
                let capacity = tx_bytes_game_socket.capacity();
                server_state.tx_bytes_gameplay_socket.store(capacity, std::sync::atomic::Ordering::Relaxed);
                tx_bytes_game_socket.send(packages).await.unwrap();
            }
        }
    });

    (rx_me_gameplay_longterm, rx_me_gameplay_webservice, rx_pe_gameplay_longterm, tx_mc_webservice_gameplay)
}