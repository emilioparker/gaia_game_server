use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use crate::{character::{character_entity::{CharacterEntity, InventoryItem}, character_reward::CharacterReward}, map::map_entity::{MapEntity, MapCommand}, ServerState, tower::{tower_entity::TowerEntity, TowerCommand}};


pub fn update_character_entity(
    player_entity : &mut CharacterEntity, 
    reward : InventoryItem,
    players_rewards_summary : &mut Vec<CharacterReward>,
    players_summary : &mut Vec<CharacterEntity>)
{
        player_entity.add_inventory_item(reward.clone());
        player_entity.version += 1;
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

pub fn report_map_process_capacity(
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    server_state : Arc<ServerState>
){
    let capacity = tx_me_gameplay_longterm.capacity();
    server_state.tx_me_gameplay_longterm.store(capacity, std::sync::atomic::Ordering::Relaxed);
    let capacity = tx_me_gameplay_webservice.capacity();
    server_state.tx_me_gameplay_webservice.store(capacity, std::sync::atomic::Ordering::Relaxed);
}

pub fn report_tower_process_capacity(
    tx_te_gameplay_longterm : &Sender<TowerEntity>,
    // tx_me_gameplay_webservice : &Sender<TowerEntity>,
    server_state : Arc<ServerState>
){
    let capacity = tx_te_gameplay_longterm.capacity();
    server_state.tx_me_gameplay_longterm.store(capacity, std::sync::atomic::Ordering::Relaxed);
    // let capacity = tx_te_gameplay_webservice.capacity();
    // server_state.tx_me_gameplay_webservice.store(capacity, std::sync::atomic::Ordering::Relaxed);
}

pub fn process_tile_attack(
    damage: &u16, 
    tile : &MapEntity, 
) -> (MapEntity, Option<InventoryItem>)
{
    // let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
    let mut updated_tile : MapEntity = tile.clone();
    let mut reward : Option<InventoryItem> = None;
    let previous_health = tile.health;
    println!("Change mob health!!! {}" ,previous_health);

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
        println!("new health {}", updated_tile.health);
        if updated_tile.health == 0
        {
            updated_tile.prop = 0;
        }

        if updated_tile.health == 0
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


pub fn get_tile_commands_to_execute(current_time : u64, delayed_tile_commands_guard : &mut Vec<(u64, MapCommand)>) -> Vec<MapCommand>
{
    let mut items_to_execute = Vec::<MapCommand>::new();
    // let current_time = time.load(std::sync::atomic::Ordering::Relaxed);

    delayed_tile_commands_guard.retain(|b| 
    {
        let should_execute = b.0 <= current_time;
        // println!("checking delayed action {} task_time {} current_time {current_time}", should_execute, b.0);
        if should_execute
        {
            items_to_execute.push(b.1.clone());
        }

        !should_execute // we keep items that we didn't execute
    });

    items_to_execute
}

pub fn get_tower_commands_to_execute(current_time : u64, delayed_tower_commands_guard : &mut Vec<(u64, TowerCommand)>) -> Vec<TowerCommand>
{
    let mut items_to_execute = Vec::<TowerCommand>::new();
    // let current_time = time.load(std::sync::atomic::Ordering::Relaxed);

    delayed_tower_commands_guard.retain(|b| 
        {
        let should_execute = b.0 <= current_time;
        // println!("checking delayed action {} task_time {} current_time {current_time}", should_execute, b.0);
        if should_execute
        {
            items_to_execute.push(b.1.clone());
        }

        !should_execute // we keep items that we didn't execute
    });

    items_to_execute
}



pub fn get_player_commands_to_execute(current_time : u64, delayed_player_commands_guards : &mut Vec<(u64, u16)>) -> Vec<u16>
{
    let mut player_commands_to_execute = Vec::<u16>::new();

    // println!("checking delayed plaeyr commands {}" , delayed_commands_lock.len());
    delayed_player_commands_guards.retain(|b| 
    {
        let should_execute = b.0 <= current_time;
        // println!("checking delayed player action {} task_time {} current_time {current_time}", should_execute, b.0);
        if should_execute
        {
            player_commands_to_execute.push(b.1);
        }

        !should_execute // we keep items that we didn't execute
    });

    player_commands_to_execute
}
