use std::{sync::Arc, collections::HashMap};
use tokio::sync::{mpsc::Sender, Mutex};
use crate::{character::{character_command::{CharacterCommand, self}, character_entity::CharacterEntity, character_presentation::CharacterPresentation, character_attack::CharacterAttack}, map::{GameMap, tetrahedron_id::TetrahedronId}};

pub async fn process_player_commands (
    map : Arc<GameMap>,
    current_time : u64,
    player_commands_processor_lock : Arc<Mutex<Vec<CharacterCommand>>>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    players_presentation_summary : &mut Vec<CharacterPresentation>,
    player_attacks_summary : &mut  Vec<CharacterAttack>,
    delayed_player_commands_lock : Arc<Mutex<Vec<(u64, u16)>>>
)
{
    let mut player_commands_data = player_commands_processor_lock.lock().await;

    if player_commands_data.len() == 0
    {
        return;
    }

    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;

    for player_command in player_commands_data.iter()
    {
        let cloned_data = player_command.to_owned();

        if let Some(atomic_time) = map.active_players.get(&cloned_data.player_id)
        {
            atomic_time.store(current_time, std::sync::atomic::Ordering::Relaxed);
        }

        if player_command.action == character_command::IDLE_ACTION 
        {
            let player_option = player_entities.get_mut(&cloned_data.player_id);
            if let Some(player_entity) = player_option 
            {
                let updated_player_entity = CharacterEntity 
                {
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
        else if player_command.action == character_command::GREET_ACTION 
        {
            let player_option = player_entities.get_mut(&cloned_data.player_id);
            if let Some(player_entity) = player_option 
            {
                let name_with_padding = format!("{: <5}", player_entity.character_name);
                let name_data : Vec<u32> = name_with_padding.chars().into_iter().map(|c| c as u32).collect();
                let mut name_array = [0u32; 5];
                name_array.clone_from_slice(&name_data.as_slice()[0..5]);
                let player_presentation = CharacterPresentation 
                {
                    player_id: player_entity.character_id,
                    character_name: name_array,
                };

                players_presentation_summary.push(player_presentation);
            }

        }
        else if player_command.action == character_command::RESPAWN_ACTION 
        { // respawn, we only update health for the moment
            let player_option = player_entities.get_mut(&cloned_data.player_id);
            if let Some(player_entity) = player_option 
            {
                let updated_player_entity = CharacterEntity 
                {
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
        else if player_command.action == character_command::WALK_ACTION 
        { // respawn, we only update health for the moment
            let player_option = player_entities.get_mut(&cloned_data.player_id);
            if let Some(player_entity) = player_option 
            {
                let updated_player_entity = CharacterEntity 
                {
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
        else if player_command.action == character_command::ATTACK_ACTION 
        { 
            // we anounce the attack
            let attack = CharacterAttack
            {
                player_id: cloned_data.player_id,
                target_player_id: cloned_data.other_player_id,
                damage: 2,
                skill_id: 0,
                target_tile_id: TetrahedronId::from_string("a0"), // we need a default value
            };
            player_attacks_summary.push(attack);

            let player_option = player_entities.get_mut(&cloned_data.player_id);
            if let Some(player_entity) = player_option 
            {
                let updated_player_entity = CharacterEntity 
                {
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
            // let current_time = time.load(std::sync::atomic::Ordering::Relaxed);
            // println!("push attack in required time {}", player_command.required_time);
            lock.push((current_time + player_command.required_time as u64, player_command.other_player_id));
        }
        else if player_command.action == character_command::ATTACK_TILE_ACTION
        || player_command.action == character_command::BUILD_ACTION 
        { // respawn, we only update health for the moment
            let player_option = player_entities.get_mut(&cloned_data.player_id);
            if let Some(player_entity) = player_option 
            {
                let updated_player_entity = CharacterEntity 
                {
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
        else if player_command.action == character_command::TOUCH 
        { 
            let player_option = player_entities.get(&cloned_data.player_id);
            if let Some(player_entity) = player_option 
            {
                players_summary.push(player_entity.clone());
            }
        }
        else 
        {
            println!("got an unknown player command {}", player_command.action)
        }
    }
    player_commands_data.clear();
    // drop(player_commands_data);
}


pub async fn process_delayed_player_commands (
    map : Arc<GameMap>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    delayed_player_commands_to_execute : Vec<u16>,
)
{
    if delayed_player_commands_to_execute.len() == 0
    {
        return;
    }

    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
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
}