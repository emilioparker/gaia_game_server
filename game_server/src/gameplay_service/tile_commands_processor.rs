use std::{sync::Arc, collections::HashMap};
use tokio::sync::{mpsc::Sender, Mutex};
use crate::{character::{character_entity::{CharacterEntity, InventoryItem}, character_attack::CharacterAttack, character_reward::CharacterReward}, map::{GameMap, map_entity::{MapCommand, MapCommandInfo, MapEntity}, tile_attack::TileAttack}, ServerState, gameplay_service::utils::update_character_entity};

use super::utils::{report_map_process_capacity, process_tile_attack};


pub async fn process_tile_commands (
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    current_time : u64,
    tile_commands_processor_lock : Arc<Mutex<Vec<MapCommand>>>,
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    tiles_summary : &mut Vec<MapEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    players_rewards_summary : &mut Vec<CharacterReward>,
    player_attacks_summary : &mut  Vec<CharacterAttack>,
    tile_attacks_summary : &mut  Vec<TileAttack>,
    delayed_tile_commands_lock : Arc<Mutex<Vec<(u64, MapCommand)>>>
)
{
    let mut tile_commands_data = tile_commands_processor_lock.lock().await;
    if tile_commands_data.len() == 0 
    {
        return;
    }

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
                        report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

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
                            updated_tile.constitution = u16::max(0, updated_tile.constitution as u16 - *damage as u16) as u16;
                            updated_tile.version += 1;
                            if updated_tile.constitution == 0
                            {
                                updated_tile.prop = 0;
                                updated_tile.health = 0;
                            }

                            tiles_summary.push(updated_tile.clone());
                            *tile = updated_tile.clone();
                            drop(tiles);

                            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                            // sending the updated tile somewhere.
                            tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                            tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                        }
                        else if previous_health > 0
                        {
                            let collected_prop = updated_tile.prop;
                            updated_tile.health = u16::max(0, updated_tile.health as u16 - *damage as u16) as u16;
                            updated_tile.version += 1;
                            if updated_tile.health == 0
                            {
                                updated_tile.prop = 0;
                                println!("updated tile is now 0");
                            }
                            tiles_summary.push(updated_tile.clone());
                            *tile = updated_tile.clone();
                            drop(tiles);

                            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                            // sending the updated tile somewhere.
                            tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                            tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();


                            if updated_tile.health == 0
                            {
                                let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
                                let player_option = player_entities.get_mut(&player_id);
                                if let Some(player_entity) = player_option 
                                {
                                    println!("Add inventory item for player");
                                    let new_item = InventoryItem 
                                    {
                                        item_id: 2, // this is to use 0 and 1 as soft and hard currency, we need to read definitions...
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
                    MapCommandInfo::LayFoundation(player_id, prop,enemy_mob, _pathness_a, _pathness_b,_pathness_c) => {

                        let current_time_in_seconds = (current_time / 1000) as u32;
                        if updated_tile.prop == 0
                        {
                            if *enemy_mob == 0
                            {
                                updated_tile.health = 500;
                                updated_tile.constitution = 0;
                            }
                            else 
                            {
                                updated_tile.health = 100;
                                updated_tile.constitution = 100;
                            }

                            updated_tile.target_id = updated_tile.id.clone();
                            updated_tile.ownership_time = current_time_in_seconds; // more seconds of control
                            updated_tile.prop = *prop;

                            let player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;

                            let player_option = player_entities.get(&player_id);
                            if let Some(player_entity) = player_option {
                                updated_tile.faction = player_entity.faction;
                            }
                            // we are creating a mob, we need to set the nature faction
                            if *enemy_mob == 1
                            {
                                updated_tile.faction = 0;
                            }

                            drop(player_entities);

                            updated_tile.version += 1;
                            tiles_summary.push(updated_tile.clone());
                            *tile = updated_tile.clone();
                            drop(tiles);

                            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

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

                            updated_tile.constitution = u16::min(updated_tile.health as u16, updated_tile.constitution as u16 + *increment as u16) as u16;
                            updated_tile.version += 1;
                            tiles_summary.push(updated_tile.clone());
                            *tile = updated_tile.clone();
                            drop(tiles);

                            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

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
                    MapCommandInfo::AttackWalker(player_id, _damage, required_time) => {

                        // updating tile stuff inmediately and releasing lock before another await.
                        updated_tile.version += 1;

                        if updated_tile.owner_id == *player_id {
                            // the controller is fighting this mob, we give him more control
                            updated_tile.ownership_time = (current_time / 1000) as u32; 
                        }
                        *tile = updated_tile.clone();
                        let tile_id = tile.id.clone();
                        let tile_level = tile.level;
                        drop(tiles);

                        let damage = if let Some(entry) = map.definitions.mob_progression.get(tile_level as usize) 
                        {
                            (entry.skill_points / 4) as u32
                        }
                        else
                        {
                            1
                        };

                        let attack = TileAttack
                        {
                            tile_id: updated_tile.id.clone(),
                            target_player_id: *player_id,
                            damage,
                            skill_id: 0,
                        };
                        tile_attacks_summary.push(attack);

                        // now we push the delayed message.

                        let mut lock = delayed_tile_commands_lock.lock().await;
                        let info = MapCommandInfo::AttackWalker(*player_id, damage as u16, *required_time);

                        let map_action = MapCommand { id: tile_id, info };
                        lock.push((current_time + *required_time as u64, map_action));
                        drop(lock);

                    },
                    MapCommandInfo::SpawnMob(player_id, mob_id, level) => {

                        if updated_tile.prop == 0 // we can spawn a mob here.
                        {
                            let current_time_in_seconds = (current_time / 1000) as u32;
                            updated_tile.level = *level as u8;

                            if let Some(entry) = map.definitions.mob_progression.get(*level as usize) 
                            {
                                let attribute = (entry.skill_points / 4) as u16;
                                updated_tile.health =  attribute;
                                updated_tile.constitution = attribute;
                                updated_tile.strength = attribute; // attack
                                updated_tile.dexterity = attribute; // attack
                            }

                            updated_tile.prop = *mob_id;
                            updated_tile.origin_id = tile.id.clone();
                            updated_tile.target_id = tile.id.clone();
                            updated_tile.faction = 4;// corruption faction
                            updated_tile.owner_id = *player_id;
                            updated_tile.ownership_time = current_time_in_seconds;

                            updated_tile.version += 1;
                            
                            // println!("new mob {:?}", updated_tile);
                            tiles_summary.push(updated_tile.clone());
                            *tile = updated_tile.clone();
                            drop(tiles);

                            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                            // sending the updated tile somewhere.
                            tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                            tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                        }
                        else {
                            tiles_summary.push(updated_tile.clone());
                        }
                    },
                    MapCommandInfo::MoveMob(player_id, mob_id, new_tile_id, _distance, required_time) => 
                    {
                        let id = tile_command.id.to_string();
                        let tile_time = updated_tile.ownership_time;
                        println!("move mob {id} tile time: {tile_time}");
                        let current_time_in_seconds = (current_time / 1000) as u32;
                        // we also need to be sure this player has control over the tile
                        if updated_tile.prop == *mob_id // we are mostly sure you know this is a mob and wants to move 
                            && &updated_tile.target_id != new_tile_id
                            && updated_tile.time < current_time_in_seconds // only if you are not doing something already
                            && updated_tile.owner_id == *player_id
                        {
                            updated_tile.version += 1;
                            // let required_time = u32::max(1, (*distance / 0.5f32).ceil() as u32);
                            let required_time = required_time.round() as u32;
                            // println!("required time {} " , required_time);
                            updated_tile.time = current_time_in_seconds + required_time;
                            updated_tile.origin_id = tile.target_id.clone();
                            updated_tile.target_id = new_tile_id.clone();

                            updated_tile.ownership_time = current_time_in_seconds; // more seconds of control
                            // println!("updating ownership time {}" , updated_tile.ownership_time);

                            tiles_summary.push(updated_tile.clone());
                            *tile = updated_tile.clone();
                            drop(tiles);

                            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                            // sending the updated tile somewhere.
                            tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                            tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                        }
                        else {
                            tiles_summary.push(updated_tile.clone());
                        }
                    },
                    MapCommandInfo::ControlMapEntity(player_id, mob_id) => {
                        let current_time_in_seconds = (current_time / 1000) as u32;
                        if updated_tile.prop == *mob_id // we are mostly sure you know this is a mob and wants to move 
                            // && updated_tile.ownership_time < current_time_in_seconds // owner timeout
                        {
                            let difference = current_time_in_seconds as i32 - updated_tile.ownership_time as i32;
                            let id = tile_command.id.to_string();
                            let tile_time = updated_tile.ownership_time;
                            // println!("for mob {id} time {current_time_in_seconds} tile time: {tile_time} difference :{difference}");

                            if difference > 60000 && *mob_id != 35 // five minutes means we should just remove it.
                            {
                                updated_tile.version += 1;
                                updated_tile.owner_id = 0;
                                updated_tile.ownership_time = 0; // seconds of control
                                updated_tile.prop = 0;
                                updated_tile.health = 0;
                                updated_tile.constitution = 0;
                            }
                            else if updated_tile.ownership_time < current_time_in_seconds 
                            {
                                // println!("updating time {current_time} {}", updated_tile.ownership_time);
                                updated_tile.version += 1;
                                updated_tile.owner_id = *player_id;
                                updated_tile.ownership_time = current_time_in_seconds; // seconds of control
                                // println!("new time {}", updated_tile.ownership_time);
                            }

                            tiles_summary.push(updated_tile.clone());
                            *tile = updated_tile.clone();
                            drop(tiles);

                            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

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
                        let info = MapCommandInfo::AttackMob(*player_id, *damage, *required_time);
                        let map_action = MapCommand { id: tile.id.clone(), info };
                        lock.push((current_time + *required_time as u64, map_action));

                        drop(lock);

                        let attack = CharacterAttack{
                            player_id: *player_id,
                            target_player_id: 0,
                            damage: *damage as u32,
                            skill_id: 0,
                            target_tile_id: tile_id.clone(),
                        };
                        player_attacks_summary.push(attack);
                    }
                    MapCommandInfo::LayWallFoundation(_player_id, faction, prop, endpoint_a, endpoint_b, wall_size) => 
                    {
                        if updated_tile.prop == 0
                        {
                            updated_tile.constitution = 0;
                            updated_tile.health = 30 * (*wall_size as u16);

                            updated_tile.origin_id = endpoint_a.clone();
                            updated_tile.target_id = endpoint_b.clone();
                            updated_tile.ownership_time = 0; // more seconds of control
                            updated_tile.prop = *prop; // it has to be a wall...

                            updated_tile.faction = *faction;


                            updated_tile.version += 1;
                            tiles_summary.push(updated_tile.clone());
                            *tile = updated_tile.clone();
                            drop(tiles);

                            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                            // sending the updated tile somewhere.
                            tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                            tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                        }
                        else {
                            tiles_summary.push(updated_tile.clone());
                        }
                    },
                }
            }
            None => println!("tile not found {}" , tile_command.id),
        }
    }
    // println!("tiles summary {} ", tiles_summary.len());
    tile_commands_data.clear();
}


pub async fn process_delayed_tile_commands (
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    tiles_summary : &mut Vec<MapEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    players_rewards_summary : &mut Vec<CharacterReward>,
    delayed_tile_commands_to_execute : Vec<MapCommand>
)
{
    for tile_command in delayed_tile_commands_to_execute.iter()
    {
        let region = map.get_region_from_child(&tile_command.id);
        let mut tiles = region.lock().await;

        match &tile_command.info 
        {
            MapCommandInfo::Touch() => todo!(),
            MapCommandInfo::ChangeHealth(_, _) => todo!(),
            MapCommandInfo::LayFoundation(_,_,_, _, _, _) => todo!(),
            MapCommandInfo::BuildStructure(_, _) => todo!(),
            MapCommandInfo::AttackWalker(player_id,damage, _required_time) => {
                drop(tiles);
                let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
                let player_option = player_entities.get_mut(&player_id);
                if let Some(player_entity) = player_option {
                    if player_entity.health > 0  
                        // && updated_tile.faction != 0 
                        // && updated_tile.faction != player_entity.faction 
                    {
                        let result = player_entity.health.saturating_sub(*damage);
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
            MapCommandInfo::SpawnMob(_, _, _) => todo!(),
            MapCommandInfo::MoveMob(_, _, _, _, _) => todo!(),
            MapCommandInfo::ControlMapEntity(_, _) => todo!(),
            MapCommandInfo::AttackMob(player_id, damage, _required_time) => 
            {
                if let Some(tile) = tiles.get_mut(&tile_command.id) {
                    let (updated_tile, reward) = process_tile_attack(
                        damage, 
                        tile, 
                    );
                    
                    *tile = updated_tile.clone();
                    drop(tiles);

                    report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                    // sending the updated tile somewhere.
                    tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                    tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
                    tiles_summary.push(updated_tile.clone());

                    if let Some(reward) = reward {
                        println!("We got some reward {:?}", reward);
                        let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
                        let player_option = player_entities.get_mut(&player_id);
                        if let Some(player_entity) = player_option {
                            update_character_entity(player_entity,reward, &map.definitions, players_rewards_summary, players_summary);
                            let updated_player_entity = player_entity.clone();
                            drop(player_entities);
                            // we try to drop any locks before doing an await
                            tx_pe_gameplay_longterm.send(updated_player_entity.clone()).await.unwrap();
                        }
                    }
                } // end of if let
            }
            MapCommandInfo::LayWallFoundation(_, _, _, _, _, _) => todo!(), // end of map command map
        }
    }
}