use std::{sync::Arc, collections::HashMap};
use tokio::sync::{mpsc::Sender, Mutex};
use crate::{ability_user::{attack::Attack, attack_result::BATTLE_MOB_MOB}, gaia_mpsc::GaiaSender, map::GameMap, tower::{tower_entity::TowerEntity, TowerCommand, TowerCommandInfo}, ServerState};
use crate::hero::{hero_entity::HeroEntity, hero_reward::HeroReward};


pub async fn process_tower_commands (
    map : Arc<GameMap>,
    _server_state: Arc<ServerState>,
    tower_commands_processor_lock : Arc<Mutex<Vec<TowerCommand>>>,
    _tx_te_gameplay_longterm : &GaiaSender<TowerEntity>,
    _tx_te_gameplay_webservice : &GaiaSender<TowerEntity>,
    _towers_summary : &mut Vec<TowerEntity>,
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
                    TowerCommandInfo::RepairTower(player_id, repair_amount) => 
                    {
                        let mut updated_tower = tower.clone();
                        let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
                        // we won't update the player, but might do it eventually. So I am paying the cost already.
                        let player_option = player_entities.get_mut(&player_id);
                        if let Some(player_entity) = player_option 
                        {
                            updated_tower.repair_damage(player_entity.faction, updated_tower.event_id, *repair_amount);
                        }

                        drop(player_entities);
                    },
                    TowerCommandInfo::AttackTower(player_id, damage, required_time) => 
                    {
                        let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
                        let current_time_in_seconds = current_time.as_secs() as u32;
                        let current_time_in_milliseconds = current_time.as_millis() as u64;

                        // tower might be sleeping or active
                        if tower.faction == 0 && tower.cooldown < current_time_in_seconds
                        {
                            let elapsed_time = current_time_in_seconds - tower.cooldown;
                            let elapsed_time_normalized = elapsed_time % (6*60);

                            if elapsed_time_normalized <= 5*60
                            {
                                // sleeping!
                                cli_log::info!("Tower is sleeping");
                            }
                            else
                            {
                                let mut lock = delayed_tower_commands_lock.lock().await;
                                let info = TowerCommandInfo::AttackTower(*player_id, *damage, *required_time);

                                let tower_action = TowerCommand { id: tower_command.id.clone(), info };
                                lock.push((current_time_in_milliseconds + *required_time as u64, tower_action));
                                drop(lock);
                            }
                        }
                        else if tower.cooldown + 10 * 60 < current_time_in_seconds // tower has been conquered it can be attacked anytime after 10 min
                        {
                            // let attack = Attack
                            // {
                            //     id : (current_time_in_milliseconds % 10000) as u16,
                            //     character_id: *player_id,
                            //     target_character_id: 0,
                            //     card_id: 0,
                            //     target_mob_tile_id: tower_command.id.clone(),
                            //     required_time : 0,
                            //     active_effect:1,
                            //     battle_type: BATTLE_MOB_MOB
                            // };
                            // player_attacks_summary.push(attack);

                            let mut lock = delayed_tower_commands_lock.lock().await;
                            let info = TowerCommandInfo::AttackTower(*player_id, *damage, *required_time);

                            let tower_action = TowerCommand { id: tower_command.id.clone(), info };
                            lock.push((current_time_in_milliseconds + *required_time as u64, tower_action));
                            drop(lock);

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
    tx_te_gameplay_longterm : &GaiaSender<TowerEntity>,
    tx_te_gameplay_webservice : &GaiaSender<TowerEntity>,
    // tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    towers_summary : &mut Vec<TowerEntity>,
    _players_summary : &mut Vec<HeroEntity>,
    _players_rewards_summary : &mut Vec<HeroReward>,
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
                TowerCommandInfo::RepairTower(_player_id, _repair_amount) => 
                {
                    // repair should not be a delayed command.
                },
                TowerCommandInfo::AttackTower(player_id, damage, _required_time) => 
                {
                    cli_log::info!("Got a tower attack");
                    let mut updated_tower = tower.clone();
                    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
                    let player_option = player_entities.get_mut(&player_id);
                    let faction_damage = if let Some(player_entity) = player_option 
                        {
                            updated_tower.add_damage_record(player_entity.faction, updated_tower.event_id, *damage)
                        }
                        else
                        {
                            0
                        };

                    drop(player_entities);

                    if faction_damage > 600 
                    {
                        // you defeated the tower!
                        updated_tower.finish_event();
                    }

                    updated_tower.version += 1;

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