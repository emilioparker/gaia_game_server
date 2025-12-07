use std::{sync::Arc, collections::HashMap};
use tokio::sync::{mpsc::Sender, Mutex};
use crate::{ability_user::{attack::Attack, attack_result::{AttackResult, BATTLE_CHAR_TOWER, BATTLE_MOB_MOB, NORMAL_ATTACK_RESULT}}, gaia_mpsc::GaiaSender, map::{tetrahedron_id::TetrahedronId, GameMap}, tower::{tower_entity::TowerEntity, TowerCommand, TowerCommandInfo}, ServerState};
use crate::hero::{hero_entity::HeroEntity, hero_reward::HeroReward};


pub async fn process_tower_commands (
    map : Arc<GameMap>,
    _server_state: Arc<ServerState>,
    tower_commands_processor_lock : Arc<Mutex<Vec<TowerCommand>>>,
    tx_te_gameplay_longterm : &GaiaSender<TowerEntity>,
    tx_te_gameplay_webservice : &GaiaSender<TowerEntity>,
    towers_summary : &mut Vec<TowerEntity>,
    player_attacks_summary : &mut  Vec<Attack>,
    delayed_tower_commands_lock : Arc<Mutex<Vec<(u64, TowerCommand)>>>
)
{
    // process tower stuff.
    let mut tower_commands_data = tower_commands_processor_lock.lock().await;
    // cli_log::info!("tower commands len {}", tower_commands_data.len());
    if tower_commands_data.len() > 0 
    {
        for tower_command in tower_commands_data.iter()
        {
            // let cloned_data = tower_command.to_owned();
            let mut towers = map.towers.lock().await;
            cli_log::info!("towers count {}", towers.len());
            let tower_option = towers.get_mut(&tower_command.id);

            if let Some(tower) = tower_option
            {
                match &tower_command.info 
                {
                    TowerCommandInfo::Touch() => todo!(),
                    TowerCommandInfo::RepairTower(player_id, faction, repair_amount) => 
                    {
                        let mut updated_tower = tower.clone();
                        if *faction == tower.faction
                        {
                            updated_tower.repair_damage(*faction, updated_tower.event_id, *repair_amount);
                            updated_tower.version += 1;
                            tx_te_gameplay_longterm.send(updated_tower.clone()).await.unwrap();
                            tx_te_gameplay_webservice.send(updated_tower.clone()).await.unwrap();
                            towers_summary.push(updated_tower.clone());

                            *tower = updated_tower;
                        }
                    },
                    TowerCommandInfo::AttackTower(player_id,event_id, player_faction, card_id, required_time) => 
                    {
                        let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
                        let current_time_in_seconds = current_time.as_secs() as u32;
                        let current_time_in_milliseconds = current_time.as_millis() as u64;

                        if tower.event_id != *event_id
                        {
                            cli_log::info!("tower event doesn't match");
                        }
                        // tower might be sleeping or active
                        else if tower.is_active(*player_faction, current_time_in_seconds)
                        {
                            // let elapsed_time = current_time_in_seconds - tower.cooldown;
                            // let elapsed_time_normalized = elapsed_time % (6*60);

                            // if elapsed_time_normalized <= 5*60
                            // {
                            //     // sleeping!
                            //     cli_log::info!("Tower is sleeping");
                            // }
                            // else
                            {
                                // let faction_damage = tower.add_damage_record(*player_faction, tower.event_id, *damage);
                                // if faction_damage > 600 
                                // {
                                //     // you defeated the tower!
                                //     tower.finish_event();
                                // }

                                // tower.version += 1;

                                // // sending the updated tile somewhere.
                                // tx_te_gameplay_longterm.send(tower.clone()).await.unwrap();
                                // tx_te_gameplay_webservice.send(tower.clone()).await.unwrap();
                                // towers_summary.push(tower.clone());
                                // *tower = updated_tower;
                            }
                        // }
                        // else
                        // {
                            let attack = Attack
                            {
                                id : (current_time_in_milliseconds % 10000) as u16,
                                attacker_hero_id: *player_id,
                                target_hero_id: 0,
                                card_id: *card_id,
                                target_tile_id: tower_command.id.clone(),
                                required_time : *required_time,
                                battle_type: crate::ability_user::attack_result::BATTLE_CHAR_TOWER,
                                attacker_mob_id: 0,
                                target_mob_id: 0,
                            };
                            player_attacks_summary.push(attack);

                            let mut lock = delayed_tower_commands_lock.lock().await;
                            let info = TowerCommandInfo::AttackTower(*player_id, *event_id, *player_faction, *card_id, *required_time);

                            let tower_action = TowerCommand { id: tower_command.id.clone(), info };
                            lock.push((current_time_in_milliseconds + *required_time as u64, tower_action));
                            drop(lock);

                        // }
                        // else
                        // {
                        }
                        else
                        {
                            cli_log::info!("Tower is in cool down");
                        }

                        // cli_log::info!("Got a tower attack towers {}", map.to);
                    },
                }
            }
            else
            {
                cli_log::info!("tower not found with id {}", tower_command.id);
            }

        }
    }
    tower_commands_data.clear();
}

pub async fn process_delayed_tower_commands (
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    current_time : u64,
    tx_te_gameplay_longterm : &GaiaSender<TowerEntity>,
    tx_te_gameplay_webservice : &GaiaSender<TowerEntity>,
    // tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    towers_summary : &mut Vec<TowerEntity>,
    _players_summary : &mut Vec<HeroEntity>,
    _players_rewards_summary : &mut Vec<HeroReward>,
    attack_details_summary : &mut Vec<AttackResult>,
    delayed_tower_commands_to_execute : Vec<TowerCommand>
)
{
    for tower_command in delayed_tower_commands_to_execute.iter()
    {
        let mut towers = map.towers.lock().await;
        cli_log::info!("towers count {}", towers.len());
        let tower_option = towers.get_mut(&tower_command.id);

        if let Some(tower) = tower_option
        {
            match &tower_command.info 
            {
                TowerCommandInfo::Touch() => todo!(),
                TowerCommandInfo::RepairTower(_player_id, faction, _repair_amount) => 
                {
                    // repair should not be a delayed command.
                },
                TowerCommandInfo::AttackTower(player_id, event_id, faction, card_id, _required_time) => 
                {
                    if *event_id == tower.event_id 
                    {
                        cli_log::info!("Got a tower attack");
                        let mut updated_tower = tower.clone();
                        let faction_damage = updated_tower.add_damage_record(*faction, updated_tower.event_id, 100);

                        if faction_damage > 600 
                        {
                            // you defeated the tower!
                            updated_tower.finish_event();
                        }

                        updated_tower.version += 1;

                        attack_details_summary.push(AttackResult
                        {
                            id: (current_time % 10000) as u16,
                            card_id : *card_id,
                            attacker_mob_id: 0,
                            attacker_character_id: *player_id,
                            target_character_id: 0,
                            target_tile_id: tower.tetrahedron_id.clone(),
                            battle_type: BATTLE_CHAR_TOWER,
                            result:NORMAL_ATTACK_RESULT,
                            target_mob_id: 0,
                        });

                        // sending the updated tile somewhere.
                        tx_te_gameplay_longterm.send(updated_tower.clone()).await.unwrap();
                        tx_te_gameplay_webservice.send(updated_tower.clone()).await.unwrap();
                        towers_summary.push(updated_tower.clone());
                        *tower = updated_tower;
                    }
                }
            }
        }

    }
}