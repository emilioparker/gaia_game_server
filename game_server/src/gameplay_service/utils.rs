use std::sync::Arc;

use rand::rngs::StdRng;
use tokio::sync::mpsc::Sender;

use crate::{ability_user::{attack_result::{BLOCKED_ATTACK_RESULT, MISSED_ATTACK_RESULT, NORMAL_ATTACK_RESULT}, AbilityUser}, buffs::buff::{BuffUser, BUFF_DEFENSE, BUFF_STRENGTH}, character::{character_command::CharacterCommand, character_entity::{CharacterEntity}, character_reward::CharacterReward}, definitions::definitions_container::Definitions, map::map_entity::{MapCommand, MapEntity}, mob::mob_command::MobCommand, tower::{tower_entity::TowerEntity, TowerCommand}, web_service::characters::PlayerCreationRequest, ServerState};


pub fn attack<T:AbilityUser+BuffUser, S:AbilityUser+BuffUser>(
    definitions : &Definitions,
    card_id:u32,
    current_time_in_seconds: u32,
    missed:u8,
    attacker: &mut T,
    target : &mut S) -> u8
{
    let attack = attacker.get_total_attack(card_id, definitions);
    attacker.use_buffs(vec![BUFF_STRENGTH], definitions);

    if missed == 1
    {
        return MISSED_ATTACK_RESULT;
    }

    let defense = target.get_total_defense( definitions);
    target.use_buffs(vec![BUFF_DEFENSE], definitions);

    let damage = attack.saturating_sub(defense);
    println!("--- attack {attack} def {defense} damage {damage}");

    let health = target.get_health();
    let updated_health = health.saturating_sub(damage);

    println!("--- attack {attack} def {defense} damage {damage} health {health} new health {updated_health}");
    target.update_health(updated_health, definitions);

    if health == updated_health
    {
        return BLOCKED_ATTACK_RESULT;
    }
    else 
    {
        if let Some(skill) = definitions.cards.get(card_id as usize)
        {
            let mut random_generator = <StdRng as rand::SeedableRng>::from_entropy();
            let x =  rand::Rng::gen::<f32>(&mut random_generator);
            if x <= skill.effect_probability 
            {
                if let Some(skill_def) = definitions.buffs.get(&skill.buff)
                {
                    target.add_buff(skill_def.code, current_time_in_seconds + 10, definitions);
                }
            }
        } 

        return NORMAL_ATTACK_RESULT;
    }
}

pub fn heal<T:AbilityUser+BuffUser, S:AbilityUser+BuffUser>(
    definitions : &Definitions,
    card_id:u32,
    current_time_in_seconds: u32,
    caster: &mut T,
    target : &mut S) -> u8
{
    target.update_health(100, definitions);
    return NORMAL_ATTACK_RESULT;
}

// pub fn add_rewards_to_character_entity(
//     player_entity : &mut CharacterEntity, 
//     reward : InventoryItem,
//     definitions : &Definitions,
//     players_rewards_summary : &mut Vec<CharacterReward>,
//     players_summary : &mut Vec<CharacterEntity>)
// {
//     player_entity.add_xp_mob_defeated(definitions);
//     player_entity.add_inventory_item(reward.clone());
//     player_entity.version += 1;
//     // we should also give the player the reward
//     let reward = CharacterReward 
//     {
//         player_id: player_entity.character_id,
//         item_id: reward.item_id,
//         amount: reward.amount,
//         inventory_hash : player_entity.inventory_version
//     };

//     println!("reward {:?}", reward);

//     players_rewards_summary.push(reward);
//     players_summary.push(player_entity.clone());
// }

pub fn report_map_process_capacity(
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    server_state : &Arc<ServerState>
){
    let capacity = tx_me_gameplay_longterm.capacity();
    server_state.tx_me_gameplay_longterm.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
    let capacity = tx_me_gameplay_webservice.capacity();
    server_state.tx_me_gameplay_webservice.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
}

pub fn report_tower_process_capacity(
    tx_te_gameplay_longterm : &Sender<TowerEntity>,
    // tx_me_gameplay_webservice : &Sender<TowerEntity>,
    server_state : Arc<ServerState>
){
    let capacity = tx_te_gameplay_longterm.capacity();
    server_state.tx_me_gameplay_longterm.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
    // let capacity = tx_te_gameplay_webservice.capacity();
    // server_state.tx_me_gameplay_webservice.store(capacity, std::sync::atomic::Ordering::Relaxed);
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

pub fn get_mob_commands_to_execute(current_time : u64, delayed_mob_commands_guard : &mut Vec<(u64, MobCommand)>) -> Vec<MobCommand>
{
    let mut items_to_execute = Vec::<MobCommand>::new();
    // let current_time = time.load(std::sync::atomic::Ordering::Relaxed);

    delayed_mob_commands_guard.retain(|b| 
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


pub fn get_player_commands_to_execute(current_time : u64, delayed_player_commands_guards : &mut Vec<(u64, CharacterCommand)>) -> Vec<CharacterCommand>
{
    let mut player_commands_to_execute = Vec::<CharacterCommand>::new();

    // println!("checking delayed plaeyr commands {}" , delayed_commands_lock.len());
    delayed_player_commands_guards.retain(|b| 
    {
        let should_execute = b.0 <= current_time;
        // println!("checking delayed player action {} task_time {} current_time {current_time}", should_execute, b.0);
        if should_execute
        {
            player_commands_to_execute.push(b.1.clone());
        }

        !should_execute // we keep items that we didn't execute
    });

    player_commands_to_execute
}
