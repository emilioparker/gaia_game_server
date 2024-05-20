use std::{sync::Arc, collections::HashMap};
use tokio::{sync::{mpsc::Sender, Mutex}, time::error::Elapsed};
use crate::{battle::{battle_command::{self, BattleCommand}, battle_instance::{self, BattleInstance}, battle_join_message::BattleJoinMessage}, character::character_entity::InventoryItem, definitions::definitions_container::Definitions, gameplay_service::utils::{process_tile_attack, update_character_entity}, map::{map_entity::MapEntity, tile_attack::TileAttack, GameMap}, tower::{tower_entity::TowerEntity, TowerCommand, TowerCommandInfo}, ServerState};
use crate::character::{character_entity::CharacterEntity, character_attack::CharacterAttack, character_reward::CharacterReward};
use rand::prelude::SliceRandom;

pub async fn process_battle_commands (
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    tx_me_gameplay_longterm : &Sender<MapEntity>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    tx_me_gameplay_webservice : &Sender<MapEntity>,
    current_time : u64,
    battle_commands_processor_lock : Arc<Mutex<Vec<BattleCommand>>>,
    battles_summary : &mut Vec<BattleInstance>,
    joins_summary : &mut  Vec<BattleJoinMessage>,
    characters_summary : &mut  Vec<CharacterEntity>,
    rewards_summary : &mut  Vec<CharacterReward>,
    map_summary : &mut  Vec<MapEntity>,
    player_attacks_summary : &mut  Vec<CharacterAttack>,
    tile_attacks_summary : &mut  Vec<TileAttack>,
)
{
    let current_time_in_seconds = (current_time / 1000) as u32;
    // process battle stuff.
    let mut battle_commands_data = battle_commands_processor_lock.lock().await;

    // println!("tower commands len {}", tower_commands_data.len());
    if battle_commands_data.len() > 0 
    {
        for battle_command in battle_commands_data.iter()
        {
            let battle_region_mutex = map.get_battle_region_from_child(&battle_command.tile_id);
            let mut battles = battle_region_mutex.lock().await;

            println!("battle count {}", battles.len());

            let create_battle = !battles.contains_key(&battle_command.tile_id);
            if create_battle
            {
                println!("creating battle with id {}", battle_command.tile_id);
                let new_battle_instance = BattleInstance ::new(battle_command.tile_id.clone(), current_time_in_seconds);
                battles.insert(battle_command.tile_id.clone(), new_battle_instance);
            }

            let battle_option = battles.get_mut(&battle_command.tile_id);


            match &battle_command.info 
            {
                battle_command::BattleCommandInfo::Touch() => 
                {

                },
                battle_command::BattleCommandInfo::Join() => 
                {
                    if let Some(battle_instance) = battle_option
                    {
                        if let Some(id) = battle_instance.join_battle(battle_command.player_id)
                        {
                            let join_message = BattleJoinMessage 
                            { 
                                target_tile_id: battle_command.tile_id.clone(),
                                player_id: battle_command.player_id,
                                participation_id: id,
                                result : 1
                            };
                            joins_summary.push(join_message);
                            battles_summary.push(battle_instance.clone());
                        }
                        else
                        {
                            let join_message = BattleJoinMessage 
                            { 
                                target_tile_id: battle_command.tile_id.clone(),
                                player_id: battle_command.player_id,
                                participation_id: 0,
                                result:0
                            };
                            joins_summary.push(join_message);
                        }
                    }
                },
                battle_command::BattleCommandInfo::Attack(participant_id, card_id) => 
                {
                    // if let Some(battle_instance) = battle_option
                    // {
                    //     // play turn
                    //     let result = battle_instance.play_turn(*participant_id, battle_command.player_id, current_time_in_seconds);
                    //     let mut updated_battle_instance = battle_instance.clone();
                    //     println!("processing attack turn {} turn: {}", result, battle_instance.turn);
                    //     // let participants : Vec<u16> = battle_instance.participants.keys().copied().collect();
                    //     drop(battles);

                    //     if result
                    //     {
                    //         let region = map.get_region_from_child(&battle_command.tile_id);
                    //         let mut tiles = region.lock().await;
                    //         let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.players.lock().await;
                    //         let character_entity_option = player_entities.get_mut(&battle_command.player_id);
                    //         let map_entity_option = tiles.get_mut(&battle_command.tile_id);

                    //         if let (Some(map_entity), Some(character_entity)) = (map_entity_option, character_entity_option)
                    //         {
                    //             // calculate attack given the card
                    //             let card_definition = map.definitions.get_card(*card_id as usize).unwrap();
                    //             let attack = card_definition.strength_factor * character_entity.get_strength() as f32;
                    //             let character_attack = attack.round() as u16;

                    //             let defense = card_definition.defense_factor * character_entity.get_defense() as f32;
                    //             let character_defense = defense.round() as u16;

                    //             println!("---------Character: {character_attack} def {character_defense}");

                    //             let tile_level = map_entity.level;
                    //             let (mob_attack, mob_defense, selected_card) = if let Some(entry) = map.definitions.mob_progression.get(tile_level as usize) 
                    //             {
                    //                 if let Some(cards) = &entry.cards
                    //                 {
                    //                     let strength = (entry.skill_points / 4) as u16;
                    //                     let selected_card = cards.choose(&mut rand::thread_rng()).unwrap();

                    //                     let card_definition = map.definitions.get_card(*selected_card as usize).unwrap();
                    //                     let damage = card_definition.strength_factor * strength as f32;
                    //                     let damage = damage.round() as u16;

                    //                     let defense = card_definition.defense_factor * strength as f32;
                    //                     let defense = defense.round() as u16;
                    //                     (damage, defense, *selected_card as u32)
                    //                 }
                    //                 else 
                    //                 {
                    //                     (1,1, 10000)
                    //                 }
                    //             }
                    //             else
                    //             {
                    //                 (1,1, 10000)
                    //             };

                    //             // record enemy card                                
                    //             // we already stored the battle instance, but we don't care about this bit of data, it is always overwriten.
                    //             updated_battle_instance.last_enemy_card_used = selected_card;
                    //             battles_summary.push(updated_battle_instance);

                    //             println!("---------Mob: {mob_attack} def {mob_defense}");
                    //             let calculated_mob_damage = character_attack.saturating_sub(mob_defense);
                    //             println!("--- mob damage {calculated_mob_damage}");
                    //             // we need to update the tile health.
                    //             // we need to check the player data.
                    //             let (updated_tile, reward) = process_tile_attack(
                    //                 // &character_entity.strength, 
                    //                 &calculated_mob_damage,
                    //                 &map_entity, 
                    //             );

                    //             *map_entity = updated_tile.clone();


                    //             drop(tiles);

                    //             if updated_tile.health == 0
                    //             {
                    //                 if let Some(reward) = reward
                    //                 {
                    //                     update_character_entity(character_entity,reward, &map.definitions, rewards_summary, characters_summary);
                    //                 }
                    //             }


                    //             let calculated_character_damage = mob_attack.saturating_sub(character_defense);
                    //             println!("--- character damage {calculated_character_damage}");

                    //             let attack = TileAttack
                    //             {
                    //                 tile_id: updated_tile.id.clone(),
                    //                 target_player_id: battle_command.player_id,
                    //                 damage: calculated_character_damage,
                    //                 skill_id: 0,
                    //             };

                    //             tile_attacks_summary.push(attack);

                    //             if character_entity.health > 0
                    //                 // && updated_tile.health > 0 // both attacks hit, doesn't matter if it kills
                    //             {
                    //                 let result = character_entity.health.saturating_sub(calculated_character_damage);
                    //                 let updated_character_entity = CharacterEntity 
                    //                 {
                    //                     action: character_entity.action,
                    //                     version: character_entity.version + 1,
                    //                     health: result,
                    //                     ..character_entity.clone()
                    //                 };

                    //                 *character_entity = updated_character_entity.clone();
                    //                 drop(player_entities);
                    //                 tx_pe_gameplay_longterm.send(updated_character_entity.clone()).await.unwrap();
                    //                 characters_summary.push(updated_character_entity.clone());
                    //             }

                    //             map_summary.push(updated_tile.clone());

                    //             crate::gameplay_service::utils::report_map_process_capacity(&tx_me_gameplay_longterm,&tx_me_gameplay_webservice, server_state.clone());

                    //             // sending the updated tile somewhere.
                    //             tx_me_gameplay_longterm.send(updated_tile.clone()).await.unwrap();
                    //             tx_me_gameplay_webservice.send(updated_tile).await.unwrap();
                    //         }
                    //     }
                    // }
                },
            }
        }
        battle_commands_data.clear();
    }
}