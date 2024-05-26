use std::{sync::Arc, collections::HashMap};
use tokio::sync::{mpsc::Sender, Mutex};
use crate::{character::{character_attack::CharacterAttack, character_entity::{CharacterEntity, InventoryItem}, character_reward::CharacterReward}, gameplay_service::utils::update_character_entity, map::{map_entity::{MapCommand, MapCommandInfo, MapEntity}, tetrahedron_id::TetrahedronId, tile_attack::TileAttack, GameMap}, ServerState};

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
        match &tile_command.info
        {
            MapCommandInfo::Touch() => 
            {
                touch(&map, &server_state, tx_me_gameplay_longterm, tx_me_gameplay_webservice, tiles_summary, tile_command.id.clone()).await;
            },
            MapCommandInfo::ResourceExtraction(player_id, damage) => 
            {
                extract_resource(&map, &server_state, tx_me_gameplay_longterm, tx_me_gameplay_webservice, tx_pe_gameplay_longterm, tiles_summary, players_summary, players_rewards_summary, *player_id, tile_command.id.clone(), *damage).await;
            }, // we need to deduct stuff from the player
            MapCommandInfo::LayFoundation(player_id, prop,enemy_mob, _pathness_a, _pathness_b,_pathness_c) => 
            {
                lay_foundation(&map, &server_state, tx_me_gameplay_longterm, tx_me_gameplay_webservice, tiles_summary, *player_id, tile_command.id.clone(), current_time, *prop).await;
            },
            MapCommandInfo::BuildStructure(_player_id, increment) => 
            {
                build_structure(&map, &server_state, tx_me_gameplay_longterm, tx_me_gameplay_webservice, tiles_summary, tile_command.id.clone(), *increment as u16).await;
            },
            MapCommandInfo::AttackWalker(player_id, damage, required_time) => 
            {
                announce_attack_walker(&map, tile_attacks_summary, tile_command.id.clone(), *player_id, current_time).await;
                if *required_time > 0
                {
                    let mut lock = delayed_tile_commands_lock.lock().await;
                    let info = MapCommandInfo::AttackWalker(*player_id, *damage, *required_time);
                    let map_action = MapCommand { id: tile_command.id.clone(), info };
                    lock.push((current_time + *required_time as u64, map_action));
                }
                else
                {
                    attack_walker(&map, &server_state, tx_pe_gameplay_longterm, players_summary, *player_id).await;
                }
            },
            MapCommandInfo::SpawnMob(player_id, mob_id, level) => 
            {
                spawn_mob(&map, &server_state, tx_me_gameplay_longterm, tx_me_gameplay_webservice, tiles_summary, tile_command.id.clone(), current_time, *player_id, *mob_id, *level as u8).await;
            },
            MapCommandInfo::MoveMob(player_id, mob_id, new_tile_id, _distance, required_time) => 
            {
                move_mob(&map, &server_state, tx_me_gameplay_longterm, tx_me_gameplay_webservice, tiles_summary, tile_command.id.clone(), new_tile_id.clone(), current_time, *required_time, *player_id, *mob_id).await;
            },
            MapCommandInfo::ControlMapEntity(player_id, mob_id) => 
            {
                control_mob(&map, &server_state, tx_me_gameplay_longterm, tx_me_gameplay_webservice, tiles_summary, tile_command.id.clone(), current_time, *player_id, *mob_id).await;
            },
            // this is very similar to change health command, but here we need to send and arrow.
            MapCommandInfo::AttackMob(player_id, card_id, required_time, active_effect) => 
            {
                let end_time = current_time + *required_time as u64;
                if *required_time == 0
                {
                    attack_mob(
                        &map,
                        &server_state,
                        tx_me_gameplay_longterm,
                        tx_me_gameplay_webservice,
                        tx_pe_gameplay_longterm,
                        tiles_summary,
                        players_summary,
                        players_rewards_summary,
                        *card_id,
                        *player_id,
                        tile_command.id.clone()).await;
                }
                else 
                {
                    println!("------------ required time for attack {required_time} current time: {current_time} {card_id}");
                    let mut lock = delayed_tile_commands_lock.lock().await;
                    let info = MapCommandInfo::AttackMob(*player_id, *card_id, *required_time, *active_effect);
                    let map_action = MapCommand { id: tile_command.id.clone(), info };
                    lock.push((end_time, map_action));
                    drop(lock);
                }

                let attack = CharacterAttack
                {
                    id: (current_time % 10000) as u16,
                    player_id: *player_id,
                    target_player_id: 0,
                    card_id: *card_id,
                    target_tile_id: tile_command.id.clone(),
                    required_time: *required_time,
                    active_effect: *active_effect
                };
                println!("--- attack {} effect {}", attack.required_time, attack.active_effect);
                player_attacks_summary.push(attack);

            }
            MapCommandInfo::LayWallFoundation(_player_id, faction, prop, endpoint_a, endpoint_b, wall_size) => 
            {
                lay_wall_foundation(&map, &server_state, tx_me_gameplay_longterm, tx_me_gameplay_webservice, tiles_summary, *prop, *faction, tile_command.id.clone(), endpoint_a.clone(), endpoint_b.clone(), *wall_size as u16).await;
            },
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
        match &tile_command.info 
        {
            MapCommandInfo::Touch() => todo!(),
            MapCommandInfo::ResourceExtraction(_, _) => todo!(),
            MapCommandInfo::LayFoundation(_,_,_, _, _, _) => todo!(),
            MapCommandInfo::BuildStructure(_, _) => todo!(),
            MapCommandInfo::AttackWalker(player_id,damage, _required_time) => 
            {
                attack_walker(&map, &server_state, tx_pe_gameplay_longterm, players_summary, *player_id).await;
            },
            MapCommandInfo::SpawnMob(_, _, _) => todo!(),
            MapCommandInfo::MoveMob(_, _, _, _, _) => todo!(),
            MapCommandInfo::ControlMapEntity(_, _) => todo!(),
            MapCommandInfo::AttackMob(player_id, card_id, _required_time, _active_effect) => 
            {
                attack_mob(
                    &map,
                    &server_state,
                    tx_me_gameplay_longterm,
                    tx_me_gameplay_webservice,
                    tx_pe_gameplay_longterm,
                    tiles_summary,
                    players_summary,
                    players_rewards_summary,
                    *card_id,
                    *player_id,
                    tile_command.id.clone()).await;
            }
            MapCommandInfo::LayWallFoundation(_, _, _, _, _, _) => todo!(), // end of map command map
        }
    }
}


pub async fn attack_mob(
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    tiles_summary : &mut Vec<MapEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    players_rewards_summary : &mut Vec<CharacterReward>,
    card_id: u32,
    player_id:u16,
    tile_id: TetrahedronId
)
{
    println!("----- attack mob ");
    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
    let player_option = player_entities.get_mut(&player_id);
    let mut character_attack = 0u16;
    if let Some(character_entity) = player_option 
    {
        if let Some(card_definition) = map.definitions.get_card(card_id as usize)
        {
            let attack = character_entity.get_strength(card_definition.strength_factor) as f32;
            character_attack = attack.round() as u16;
            character_entity.use_buffs(vec![crate::character::character_entity::Stat::Strength]);
        }
        else
        {
            println!("-- card id not found {card_id}");
            return;
        }
    }

    drop(player_entities);

    println!("----- attack mob {character_attack}");
    let region = map.get_region_from_child(&tile_id);
    let mut tiles = region.lock().await;

    if let Some(tile) = tiles.get_mut(&tile_id) 
    {
        let (updated_tile, reward) = process_tile_attack(
            character_attack, 
            tile, 
        );
        
        *tile = updated_tile.clone();
        drop(tiles);

        report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, &server_state);

        // sending the updated tile somewhere.
        tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
        tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
        tiles_summary.push(updated_tile.clone());

        if let Some(reward) = reward 
        {
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

pub async fn touch(
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    tiles_summary : &mut Vec<MapEntity>,
    tile_id: TetrahedronId
)
{
    let region = map.get_region_from_child(&tile_id);
    let mut tiles = region.lock().await;
    if let Some(tile) = tiles.get_mut(&tile_id)
    {
        tiles_summary.push(tile.clone());
        drop(tiles);
        report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, &server_state);
    }
}

pub async fn extract_resource(
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    tiles_summary : &mut Vec<MapEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    players_rewards_summary : &mut Vec<CharacterReward>,
    player_id:u16,
    tile_id: TetrahedronId,
    damage: u16
)
{                       
    let region = map.get_region_from_child(&tile_id);
    let mut tiles = region.lock().await;
    if let Some(tile) = tiles.get_mut(&tile_id)
    {
        let mut updated_tile = tile.clone();
        let previous_health = tile.health;
        println!("Change tile health!!! {}", tile.prop);
        // this means this tile is being built
        if tile.health > tile.constitution 
        {
            updated_tile.constitution = u16::max(0, updated_tile.constitution as u16 - damage) as u16;
            updated_tile.version += 1;
            if updated_tile.constitution == 0
            {
                updated_tile.prop = 0;
                updated_tile.health = 0;
            }

            tiles_summary.push(updated_tile.clone());
            *tile = updated_tile.clone();
            drop(tiles);

            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, &server_state);

            // sending the updated tile somewhere.
            tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
            tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
        }
        else if previous_health > 0
        {
            let collected_prop = updated_tile.prop;
            updated_tile.health = u16::max(0, updated_tile.health as u16 - damage) as u16;
            updated_tile.version += 1;
            if updated_tile.health == 0
            {
                updated_tile.prop = 0;
                println!("updated tile is now 0");
            }
            tiles_summary.push(updated_tile.clone());
            *tile = updated_tile.clone();
            drop(tiles);

            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, &server_state);

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
                        equipped: 0,
                        amount: 1,
                    };

                    player_entity.add_inventory_item(new_item.clone());
                    player_entity.version += 1;

                    let updated_player_entity = player_entity.clone();

                    drop(player_entities);
                    // we should also give the player the reward
                    let reward = CharacterReward 
                    {
                        player_id,
                        item_id: new_item.item_id,
                        amount: new_item.amount,
                        inventory_hash : updated_player_entity.inventory_version
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
    }
}

pub async fn lay_foundation(
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    tiles_summary : &mut Vec<MapEntity>,
    player_id:u16,
    tile_id: TetrahedronId,
    current_time : u64,
    prop: u32
)
{
    let current_time_in_seconds = (current_time / 1000) as u32;
    let region = map.get_region_from_child(&tile_id);
    let mut tiles = region.lock().await;
    if let Some(tile) = tiles.get_mut(&tile_id)
    {
        let mut updated_tile = tile.clone();
        if updated_tile.prop == 0
        {
            updated_tile.health = 500;
            updated_tile.constitution = 0;

            updated_tile.target_id = updated_tile.id.clone();
            updated_tile.ownership_time = current_time_in_seconds; // more seconds of control
            updated_tile.prop = prop;

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

            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, &server_state);

            // sending the updated tile somewhere.
            tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
            tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
        }
        else 
        {
            tiles_summary.push(updated_tile.clone());
        }
    }

}

pub async fn build_structure(
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    tiles_summary : &mut Vec<MapEntity>,
    tile_id: TetrahedronId,
    increment: u16
)
{
    let region = map.get_region_from_child(&tile_id);
    let mut tiles = region.lock().await;
    if let Some(tile) = tiles.get_mut(&tile_id)
    {
        let mut updated_tile = tile.clone();
        if updated_tile.health > updated_tile.constitution 
        {

            updated_tile.constitution = u16::min(updated_tile.health as u16, updated_tile.constitution as u16 + increment as u16) as u16;
            updated_tile.version += 1;
            tiles_summary.push(updated_tile.clone());
            *tile = updated_tile.clone();
            drop(tiles);

            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, &server_state);

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
    }

}

pub async fn announce_attack_walker(
    map : &Arc<GameMap>,
    tile_attacks_summary : &mut Vec<TileAttack>,
    tile_id: TetrahedronId,
    player_id: u16,
    current_time : u64
)
{
    let region = map.get_region_from_child(&tile_id);
    let mut tiles = region.lock().await;
    if let Some(tile) = tiles.get_mut(&tile_id)
    {
        let mut updated_tile = tile.clone();
        // updating tile stuff inmediately and releasing lock before another await.
        updated_tile.version += 1;

        if updated_tile.owner_id == player_id {
            // the controller is fighting this mob, we give him more control
            updated_tile.ownership_time = (current_time / 1000) as u32; 
        }
        *tile = updated_tile.clone();
        let tile_id = tile.id.clone();
        let tile_level = tile.level;
        drop(tiles);

        let damage = if let Some(entry) = map.definitions.mob_progression.get(tile_level as usize) 
        {
            (entry.skill_points / 4) as u16
        }
        else
        {
            1
        };

        let attack = TileAttack
        {
            tile_id: updated_tile.id.clone(),
            target_player_id: player_id,
            damage,
            skill_id: 0,
        };
        tile_attacks_summary.push(attack);

        // now we push the delayed message.

    }
}

pub async fn attack_walker(
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    player_id: u16
)
{
    // drop(tiles);
    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
    let player_option = player_entities.get_mut(&player_id);
    if let Some(player_entity) = player_option 
    {
        let damage = 5u16;
        if player_entity.health > 0  
            // && updated_tile.faction != 0 
            // && updated_tile.faction != player_entity.faction 
        {
            let result = player_entity.health.saturating_sub(damage);
            let updated_player_entity = CharacterEntity 
            {
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
}

pub async fn spawn_mob(
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    tiles_summary : &mut Vec<MapEntity>,
    tile_id: TetrahedronId,
    current_time : u64,
    player_id: u16,
    mob_id: u32,
    level: u8
)
{
    let region = map.get_region_from_child(&tile_id);
    let mut tiles = region.lock().await;
    if let Some(tile) = tiles.get_mut(&tile_id)
    {
        let mut updated_tile = tile.clone();
        if updated_tile.prop == 0 // we can spawn a mob here.
        {
            let current_time_in_seconds = (current_time / 1000) as u32;
            updated_tile.level = level;

            if let Some(entry) = map.definitions.mob_progression.get(level as usize) 
            {
                let attribute = (entry.skill_points / 4) as u16;
                updated_tile.health =  attribute;
                updated_tile.constitution = attribute;
                updated_tile.strength = attribute; // attack
                updated_tile.dexterity = attribute; // attack
            }

            updated_tile.prop = mob_id;
            updated_tile.origin_id = tile.id.clone();
            updated_tile.target_id = tile.id.clone();
            updated_tile.faction = 4;// corruption faction
            updated_tile.owner_id = player_id;
            updated_tile.ownership_time = current_time_in_seconds;

            updated_tile.version += 1;
            
            // println!("new mob {:?}", updated_tile);
            tiles_summary.push(updated_tile.clone());
            *tile = updated_tile.clone();
            drop(tiles);

            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, &server_state);

            // sending the updated tile somewhere.
            tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
            tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
        }
        else 
        {
            tiles_summary.push(updated_tile.clone());
        }
    }
}

pub async fn move_mob(
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    tiles_summary : &mut Vec<MapEntity>,
    tile_id: TetrahedronId,
    new_tile_id: TetrahedronId,
    current_time : u64,
    required_time : f32,
    player_id: u16,
    mob_id: u32,
)
{
    let region = map.get_region_from_child(&tile_id);
    let mut tiles = region.lock().await;
    if let Some(tile) = tiles.get_mut(&tile_id)
    {
        let mut updated_tile = tile.clone();
        let id = tile_id.to_string();
        let tile_time = updated_tile.ownership_time;
        println!("move mob {id} tile time: {tile_time}");
        let current_time_in_seconds = (current_time / 1000) as u32;
        // we also need to be sure this player has control over the tile
        if updated_tile.prop == mob_id // we are mostly sure you know this is a mob and wants to move 
            && updated_tile.target_id != new_tile_id
            && updated_tile.time < current_time_in_seconds // only if you are not doing something already
            && updated_tile.owner_id == player_id
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

            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, &server_state);

            // sending the updated tile somewhere.
            tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
            tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
        }
        else 
        {
            tiles_summary.push(updated_tile.clone());
        }
    }

}

pub async fn control_mob(
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    tiles_summary : &mut Vec<MapEntity>,
    tile_id: TetrahedronId,
    current_time : u64,
    player_id: u16,
    mob_id: u32
)
{
    let region = map.get_region_from_child(&tile_id);
    let mut tiles = region.lock().await;
    if let Some(tile) = tiles.get_mut(&tile_id)
    {
        let mut updated_tile = tile.clone();
        let current_time_in_seconds = (current_time / 1000) as u32;
        if updated_tile.prop == mob_id // we are mostly sure you know this is a mob and wants to move 
            // && updated_tile.ownership_time < current_time_in_seconds // owner timeout
        {
            let difference = current_time_in_seconds as i32 - updated_tile.ownership_time as i32;
            // let id = tile_command.id.to_string();
            // let tile_time = updated_tile.ownership_time;
            // println!("for mob {id} time {current_time_in_seconds} tile time: {tile_time} difference :{difference}");

            if difference > 60000 && mob_id != 35 // five minutes means we should just remove it.
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
                updated_tile.owner_id = player_id;
                updated_tile.ownership_time = current_time_in_seconds; // seconds of control
                // println!("new time {}", updated_tile.ownership_time);
            }

            tiles_summary.push(updated_tile.clone());
            *tile = updated_tile.clone();
            drop(tiles);

            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, &server_state);

            // sending the updated tile somewhere.
            tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
            tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
        }
        else 
        {
            // println!("Somethign failed {current_time} {}", updated_tile.ownership_time);
            tiles_summary.push(updated_tile.clone());
        }
    }

}

pub async fn lay_wall_foundation(
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    tiles_summary : &mut Vec<MapEntity>,
    prop:u32,
    faction:u8,
    tile_id: TetrahedronId,
    endpoint_a: TetrahedronId,
    endpoint_b: TetrahedronId,
    wall_size: u16,
)
{
    let region = map.get_region_from_child(&tile_id);
    let mut tiles = region.lock().await;
    if let Some(tile) = tiles.get_mut(&tile_id)
    {
        let mut updated_tile = tile.clone();
        if updated_tile.prop == 0
        {
            updated_tile.constitution = 0;
            updated_tile.health = 30 * wall_size;

            updated_tile.origin_id = endpoint_a.clone();
            updated_tile.target_id = endpoint_b.clone();
            updated_tile.ownership_time = 0; // more seconds of control
            updated_tile.prop = prop; // it has to be a wall...
            updated_tile.faction = faction;
            updated_tile.version += 1;
            tiles_summary.push(updated_tile.clone());
            *tile = updated_tile.clone();
            drop(tiles);

            report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, &server_state);

            // sending the updated tile somewhere.
            tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
            tx_me_gameplay_webservice.send(updated_tile.clone()).await.unwrap();
        }
        else
        {
            tiles_summary.push(updated_tile.clone());
        }
    }

}