use std::{collections::HashMap, sync::Arc, u16};
use rand::rngs::StdRng;
use tokio::sync::{mpsc::Sender, Mutex};
use crate::{ServerState, ability_user::{attack::Attack, attack_result::{AttackResult, BATTLE_CHAR_MOB, BATTLE_MOB_CHAR, BATTLE_MOB_MOB}}, buffs::buff::BuffUser, definitions::definitions_container::Definitions, gaia_mpsc::GaiaSender, hero::{hero_entity::{INSIDE_TOWER_FLAG, TRYING_TO_ENTER_TOWER_FLAG}, hero_inventory::InventoryItem}, map::{GameMap, tetrahedron_id::{self, TetrahedronId}}, mob::{mob_command::{self, MobCommand}, mob_entity::MobEntity}};
use crate::hero::{hero_entity::HeroEntity, hero_reward::HeroReward};

pub async fn process_mob_commands (
    map : Arc<GameMap>,
    current_time : u64,
    server_state: Arc<ServerState>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    tx_moe_gameplay_webservice : &GaiaSender<MobEntity>,
    mobs_commands_processor_lock : Arc<Mutex<Vec<MobCommand>>>,
    delayed_mob_commands_lock : Arc<Mutex<Vec<(u64, MobCommand)>>>,
    mobs_summary : &mut Vec<MobEntity>,
    characters_summary : &mut  Vec<HeroEntity>,
    attack_details_summary : &mut Vec<AttackResult>,
    rewards_summary : &mut  Vec<HeroReward>,
    attacks_summary : &mut  Vec<Attack>,
)
{
    let mut mobs_commands_data = mobs_commands_processor_lock.lock().await;

    // cli_log::info!("mobs commands len {}", mobs_commands_data.len());
    if mobs_commands_data.len() > 0 
    {
        for mobs_command in mobs_commands_data.iter()
        {
            match mobs_command
            {
                mob_command::MobCommand::Touch(data) => 
                {
                    check_buffs(
                        &map,
                        current_time,
                        &server_state,
                        tx_moe_gameplay_webservice,
                        mobs_summary, 
                        data.mob_id,
                        data.mob_tile_id.clone()
                    ).await;
                },
                mob_command::MobCommand::CastFromMobToMob(data) => 
                {
                    let end_time = current_time + data.time as u64;
                    if data.time == 0
                    {
                        cast_mob_from_mob(
                            &map,
                            current_time,
                            &server_state,
                            tx_moe_gameplay_webservice,
                            mobs_summary,
                            attack_details_summary,
                            data.card_id,
                            data.caster_mob_tile_id.clone(),
                            data.caster_mob_id,
                            data.target_mob_tile_id.clone(),
                            data.target_mob_id,
                            data.missed
                        ).await;
                    }
                    else
                    {
                        // cli_log::info!("------------ required time for cast to mob {data.time} current time: {current_time} {card_id}");
                        let mut lock = delayed_mob_commands_lock.lock().await;
                        let delayed_command = mob_command::MobCommand::CastFromMobToMob(data.clone());
                        // let mob_action = MobCommand { tile_id : mobs_command.tile_id.clone(), info };
                        lock.push((end_time, delayed_command));
                        drop(lock);

                        // we only send attack messages if attack is delayed, for projectiles and other instances.
                        let attack = Attack
                        {
                            id: (current_time % 10000) as u16,
                            attacker_hero_id: 0,
                            target_hero_id: 0,
                            card_id: data.card_id,
                            required_time: data.time,
                            battle_type : BATTLE_MOB_MOB,
                            attacker_mob_id: data.caster_mob_id,
                            target_tile_id: TetrahedronId::default(),
                            target_mob_id: data.target_mob_id,
                        };

                        cli_log::info!("--- cast {} ", attack.required_time);
                        attacks_summary.push(attack);
                    }

                },
                mob_command::MobCommand::CastFromHeroToMob(data) => 
                {
                    let end_time = current_time + data.time as u64;
                    if data.time == 0
                    {
                        cast_mob_from_character(
                            &map,
                            current_time,
                            &server_state,
                            tx_moe_gameplay_webservice,
                            tx_pe_gameplay_longterm,
                            mobs_summary,
                            characters_summary,
                            attack_details_summary,
                            rewards_summary,
                            data.card_id,
                            data.hero_id,
                            data.target_mob_id,
                            data.target_mob_tile_id.clone(),
                            data.missed,
                        ).await;
                    }
                    else 
                    {
                        // cli_log::info!("------------ required time for attack to mob {required_time} current time: {current_time} {card_id}");
                        let mut lock = delayed_mob_commands_lock.lock().await;
                        let delayed_command = mob_command::MobCommand::CastFromHeroToMob(data.clone());
                        lock.push((end_time, delayed_command));
                        drop(lock);

                        // we only send attack messages if attack is delayed, for projectiles and other instances.
                        let attack = Attack
                        {
                            id: (current_time % 10000) as u16,
                            attacker_hero_id: data.hero_id,
                            target_hero_id: 0,
                            attacker_mob_id: 0,
                            target_mob_id: data.target_mob_id,
                            card_id: data.card_id,
                            required_time: data.time,
                            target_tile_id: TetrahedronId::default(),
                            battle_type : BATTLE_CHAR_MOB,
                        };

                        cli_log::info!("--- attack {}", attack.required_time);
                        attacks_summary.push(attack);
                    }

                },
                mob_command::MobCommand::Spawn(data) => 
                {
                    spawn_mob(&map, &server_state, tx_moe_gameplay_webservice, mobs_summary, data.tile_id.clone(), current_time, data.hero_id, data.mob_definition_id, data.level).await;
                },
                mob_command::MobCommand::ControlMob(data) => 
                {
                    control_mob(&map, &server_state, tx_moe_gameplay_webservice, mobs_summary, data.mob_id, data.mob_tile_id.clone(), current_time, data.hero_id).await;
                },
                mob_command::MobCommand::MoveMob(data) => 
                {
                    move_mob(&map, &server_state, tx_moe_gameplay_webservice, mobs_summary, current_time, data.hero_id, data.mob_id, data.new_origin_tile_id.clone(), data.new_end_tile_id.clone(), data.path).await;
                },
                mob_command::MobCommand::AttackFromMobToHero(data) => 
                {
                    let end_time = current_time + data.time as u64;
                    if data.time == 0
                    {
                        cast_hero_from_mob(
                            &map,
                            current_time,
                            &server_state,
                            tx_moe_gameplay_webservice,
                            tx_pe_gameplay_longterm,
                            mobs_summary,
                            characters_summary,
                            attack_details_summary,
                            data.card_id,
                            data.hero_id,
                            data.attacker_mob_id,
                            data.attacker_mob_tile_id.clone(),
                            data.missed,
                        ).await;
                    }
                    else 
                    {
                        // cli_log::info!("------------ required time for attack to character from mob {required_time} current time: {current_time} {card_id}");
                        let mut lock = delayed_mob_commands_lock.lock().await;
                        let delayed_command = mob_command::MobCommand::AttackFromMobToHero(data.clone());
                        lock.push((end_time, delayed_command));
                        drop(lock);

                        // we only send attack messages if attack is delayed, for projectiles and other instances.
                        let attack = Attack
                        {
                            id: (current_time % 10000) as u16,
                            attacker_hero_id: 0,
                            target_hero_id: data.hero_id,
                            attacker_mob_id: data.attacker_mob_id,
                            target_mob_id: 0,
                            card_id: data.card_id,
                            required_time: data.time,
                            target_tile_id: TetrahedronId::default(),
                            battle_type : BATTLE_MOB_CHAR,
                        };

                        cli_log::info!("--- attack {} ", attack.required_time);
                        attacks_summary.push(attack);
                    }
                },
            }
        }
        mobs_commands_data.clear();
    }
}

pub async fn process_delayed_mob_commands (
    map : Arc<GameMap>,
    current_time : u64,
    server_state: Arc<ServerState>,
    tx_moe_gameplay_webservice : &GaiaSender<MobEntity>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    mobs_summary : &mut Vec<MobEntity>,
    characters_summary : &mut Vec<HeroEntity>,
    attack_details_summary : &mut Vec<AttackResult>,
    rewards_summary : &mut Vec<HeroReward>,
    delayed_mob_commands_to_execute : Vec<MobCommand>
)
{
    for mobs_command in delayed_mob_commands_to_execute.iter()
    {
        match mobs_command
        {
            mob_command::MobCommand::Touch(_) => todo!(),
            mob_command::MobCommand::Spawn(_) => todo!(),
            mob_command::MobCommand::ControlMob(_) => todo!(),
            mob_command::MobCommand::MoveMob(_) => todo!(),
            mob_command::MobCommand::CastFromMobToMob(data) => 
            {
                cast_mob_from_mob(
                    &map,
                    current_time,
                    &server_state,
                    tx_moe_gameplay_webservice,
                    mobs_summary,
                    attack_details_summary,
                    data.card_id,
                    data.caster_mob_tile_id.clone(),
                    data.caster_mob_id,
                    data.target_mob_tile_id.clone(),
                    data.target_mob_id,
                    data.missed,
                ).await;
            }
            mob_command::MobCommand::CastFromHeroToMob(data) => 
            {
                cast_mob_from_character(
                    &map,
                    current_time,
                    &server_state,
                    tx_moe_gameplay_webservice,
                    tx_pe_gameplay_longterm,
                    mobs_summary,
                    characters_summary,
                    attack_details_summary,
                    rewards_summary,
                    data.card_id,
                    data.hero_id,
                    data.target_mob_id,
                    data.target_mob_tile_id.clone(),
                    data.missed,
                ).await;
            },
            mob_command::MobCommand::AttackFromMobToHero(data) => 
            {
                cast_hero_from_mob(
                    &map,
                    current_time,
                    &server_state,
                    tx_moe_gameplay_webservice,
                    tx_pe_gameplay_longterm,
                    mobs_summary,
                    characters_summary,
                    attack_details_summary,
                    data.card_id,
                    data.hero_id,
                    data.attacker_mob_id,
                    data.attacker_mob_tile_id.clone(),
                    data.missed,
                ).await;
            },
        }
    }
}

pub async fn spawn_mob(
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    tx_moe_gameplay_webservice : &GaiaSender<MobEntity>,
    // tx_me_gameplay_longterm : &Sender<MapEntity>,
    // tx_me_gameplay_webservice : &Sender<MapEntity>,
    mobs_summary : &mut Vec<MobEntity>,
    tile_id: TetrahedronId,
    current_time : u64,
    character_id: u16,
    definition_id: u32,
    level: u8
)
{
    let current_time_in_seconds = (current_time / 1000) as u32;
    let mut new_mob = MobEntity
    {
        mob_id : 0,
        mob_definition_id: definition_id as u16,
        level,
        version: 0,
        owner_id: character_id,
        ownership_time: current_time_in_seconds,
        start_position_id : tile_id.clone(),
        end_position_id : tile_id.clone(),
        path: [0,0,0,0,0,0],
        time: 0,
        health: 0,
        buffs: Vec::new(),
        buffs_summary: [0,0,0,0,0],
    };


    if let Some(mob_progression) = map.definitions.mob_progression_by_mob.get(definition_id as usize)
    {
        if let Some(entry) = mob_progression.get(level as usize) 
        {
            new_mob.health =  entry.constitution;
        }
    }

    let region = map.get_mob_region_from_child(&tile_id);
    let region_for_positions = map.get_mob_positions_region_from_child(&tile_id);
    let mut mobs = region.lock().await;
    let mut mob_positions = region_for_positions.lock().await;

    if mob_positions.contains(&tile_id) 
    {
        // this place is already occupied
        cli_log::info!("Move position is occupied {tile_id}");
        return;
    }


    // if let Some(mob) = mobs.get_mut(&tile_id)
    // {
    //     if mob.health <= 0 // we can spawn a mob here.
    //     {
    //         new_mob.version = mob.version + 1;
    //         // cli_log::info!("new mob {:?}", updated_tile);
    //         *mob = new_mob.clone();
    //         mob_positions.insert(tile_id);
    //         drop(mobs);
    //         drop(mob_positions);
    //         mobs_summary.push(new_mob.clone());
    //         tx_moe_gameplay_webservice.send(new_mob).await.unwrap();
    //     }
    //     else 
    //     {
    //         mobs_summary.push(mob.clone());
    //         drop(mobs);
    //         drop(mob_positions);
    //     }
    // }
    // else
    {
        let new_id = server_state.mob_id_generator.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        new_mob.mob_id = new_id;
        mobs_summary.push(new_mob.clone());
        mobs.insert(new_id, new_mob.clone());
        mob_positions.insert(tile_id);
        drop(mobs);
        drop(mob_positions);

        tx_moe_gameplay_webservice.send(new_mob).await.unwrap();
    }
}

pub async fn control_mob(
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    tx_moe_gameplay_webservice : &GaiaSender<MobEntity>,
    mobs_summary : &mut Vec<MobEntity>,
    mob_id: u32,
    mob_tile_id: TetrahedronId,
    current_time : u64,
    player_id: u16
)
{
    let mob_region = map.get_mob_region_from_child(&mob_tile_id);
    let mut mobs = mob_region.lock().await;
    if let Some(mob) = mobs.get_mut(&mob_id)
    {
        let mut updated_mob = mob.clone();
        let current_time_in_seconds = (current_time / 1000) as u32;
        let difference = current_time_in_seconds as i32 - updated_mob.ownership_time as i32;
        // let id = tile_command.id.to_string();
        // let tile_time = updated_tile.ownership_time;
        // cli_log::info!("for mob {id} time {current_time_in_seconds} tile time: {tile_time} difference :{difference}");

        if difference > 60000
        {
            updated_mob.version += 1;
            updated_mob.owner_id = 0;
            updated_mob.ownership_time = 0; // seconds of control
            updated_mob.mob_definition_id = 0;
            updated_mob.health = 0;
            // updated_mob.constitution = 0;
        }
        else if updated_mob.ownership_time < current_time_in_seconds 
        {
            // cli_log::info!("updating time {current_time} {}", updated_tile.ownership_time);
            updated_mob.version += 1;
            updated_mob.owner_id = player_id;
            updated_mob.ownership_time = current_time_in_seconds; // seconds of control
            // cli_log::info!("new time {}", updated_tile.ownership_time);
        }

        mobs_summary.push(updated_mob.clone());
        *mob = updated_mob.clone();
        drop(mobs);

        // sending the updated tile somewhere.
        tx_moe_gameplay_webservice.send(updated_mob.clone()).await.unwrap();
    }
}

pub async fn move_mob(
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    tx_moe_gameplay_webservice : &GaiaSender<MobEntity>,
    mobs_summary : &mut Vec<MobEntity>,
    current_time : u64,
    hero_id: u16,
    mob_id : u32,
    new_origin_position_id: TetrahedronId,
    new_end_position_id: TetrahedronId,
    path: [u8;6],
)
{
    let mob_region = map.get_mob_region_from_child(&new_end_position_id);
    let previous_region_id = TetrahedronId::get_parent(&new_origin_position_id, 7);
    let new_region_id = TetrahedronId::get_parent(&new_end_position_id, 7);

    let region_for_positions = map.get_mob_positions_region_from_child(&new_end_position_id);

    let mut mobs = mob_region.lock().await;
    let mut mob_positions = region_for_positions.lock().await;

    let occupied_spot = mob_positions.contains(&new_end_position_id);

    if occupied_spot
    {
        return;
    }


    if let Some(mob) = mobs.get_mut(&mob_id)
    {
        let mut updated_mob = mob.clone();
        let current_time_in_seconds = (current_time / 1000) as u32;

        if updated_mob.health == 0 || updated_mob.owner_id != hero_id || updated_mob.end_position_id != new_origin_position_id
        {
            drop(mobs);
            tx_moe_gameplay_webservice.send(updated_mob).await.unwrap();
            return;
        }

        // somehow the compiler know that this lock is not the same as the above, there is not way for a deadlock to happen
        if new_region_id != previous_region_id
        {
            let region_for_positions = map.get_mob_positions_region_from_child(&previous_region_id);
            let mut mob_positions = region_for_positions.lock().await;
            mob_positions.remove(&new_origin_position_id);
            drop(mob_positions);
        }

        mob_positions.remove(&new_origin_position_id);
        mob_positions.insert(new_end_position_id.clone());

        // let previous_end_position = updated_mob.end_position_id;
        updated_mob.version += 1;
        updated_mob.start_position_id = new_origin_position_id;
        updated_mob.end_position_id = new_end_position_id;
        updated_mob.path = path;
        updated_mob.time = current_time_in_seconds;

        mobs_summary.push(updated_mob.clone());

        *mob = updated_mob.clone();

        drop(mobs);
        drop(mob_positions);

        // sending the updated tile somewhere.
        tx_moe_gameplay_webservice.send(updated_mob.clone()).await.unwrap();
    }
}

pub async fn cast_mob_from_mob(
    map : &Arc<GameMap>,
    current_time : u64,
    server_state: &Arc<ServerState>,
    tx_moe_gameplay_webservice : &GaiaSender<MobEntity>,
    mobs_summary : &mut Vec<MobEntity>,
    attack_details_summary : &mut Vec<AttackResult>,
    card_id: u32,
    caster_mob_tile_id: TetrahedronId,
    caster_mob_id : u32,
    target_mob_tile_id: TetrahedronId,
    target_mob_id : u32,
    missed: u8,
)
{
    cli_log::info!("----- cast to mob from mob ");

    // let key = self.get_parent(tetrahedron_id);

    let mob_region = map.get_mob_region_from_child(&caster_mob_tile_id);
    let mut mobs = mob_region.lock().await;
    let mob_caster_option = mobs.get(&caster_mob_id);
    let mob_target_option = mobs.get(&target_mob_id);

    let current_time_in_seconds = (current_time / 1000) as u32;
    if let (Some(mob_caster), Some(mob_target)) = (mob_caster_option, mob_target_option)
    {
        let mut caster = mob_caster.clone();
        let mut target = mob_target.clone();
        cli_log::info!("---- calling heal {} id: {}" , mob_target.health, mob_target.mob_id);
        let result = super::utils::heal::<MobEntity, MobEntity>(&map.definitions, card_id, current_time_in_seconds, &mut caster, &mut target);

        caster.version += 1;
        target.version += 1;
        
        let mob_caster_stored = caster.clone();
        let mob_target_stored = target.clone();

        if let Some(mob_caster)= mobs.get_mut(&caster_mob_id)
        {
            *mob_caster = caster;
        }

        if let Some(mob_target)= mobs.get_mut(&target_mob_id)
        {
            *mob_target = target;
        }

        drop(mobs);

        attack_details_summary.push(AttackResult
        {
            id: (current_time % 10000) as u16,
            card_id,
            attacker_mob_id:caster_mob_id,
            attacker_character_id: 0,
            target_character_id: 0,
            target_mob_id : target_mob_id,
            target_tile_id: TetrahedronId::default(),
            battle_type: BATTLE_MOB_MOB,
            result,
        });

        mobs_summary.push(mob_target_stored.clone());
        mobs_summary.push(mob_caster_stored.clone());

        tx_moe_gameplay_webservice.send(mob_caster_stored).await.unwrap();
        tx_moe_gameplay_webservice.send(mob_target_stored).await.unwrap();
    }
}

pub async fn cast_mob_from_character(
    map : &Arc<GameMap>,
    current_time : u64,
    server_state: &Arc<ServerState>,
    tx_moe_gameplay_webservice : &GaiaSender<MobEntity>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    mobs_summary : &mut Vec<MobEntity>,
    characters_summary : &mut Vec<HeroEntity>,
    attack_details_summary : &mut Vec<AttackResult>,
    characters_rewards_summary : &mut Vec<HeroReward>,
    card_id: u32,
    character_id:u16,
    mob_id: u32,
    mob_tile_id: TetrahedronId,
    missed: u8,
)
{
    cli_log::info!("----- attack mob ");
    let mut character_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
    let character_attacker_option = character_entities.get(&character_id);

    let mob_region = map.get_mob_region_from_child(&mob_tile_id);
    let mut mobs = mob_region.lock().await;

    let mob_defender_option = mobs.get(&mob_id);
    
    let current_time_in_seconds = (current_time / 1000) as u32;
    if let (Some(attacker), Some(defender)) = (character_attacker_option, mob_defender_option)
    {
        let mut attacker = attacker.clone();
        let mut defender = defender.clone();
        let result = super::utils::attack::<HeroEntity, MobEntity>(&map.definitions, card_id, current_time_in_seconds, missed, &mut attacker, &mut defender);

        attacker.version += 1;
        defender.version += 1;

        if defender.health <= 0 
        {
            let base_xp = defender.level + 1;
            let factor = 1.1f32.powf((defender.level as i32 - attacker.level as i32).max(0) as f32);
            let xp = base_xp as f32 * factor;

            cli_log::info!("base_xp:{base_xp} - factor:{factor} xp: {xp}");

            attacker.add_xp_from_battle(xp.ceil() as u32, &map.definitions);
            let reward = InventoryItem 
            {
                item_id: 2, // this is to use 0 and 1 as soft and hard currency, we need to read definitions...
                equipped:0,
                amount: 1,
            };
            attacker.add_inventory_item(reward);


            let mut random_generator = <StdRng as rand::SeedableRng>::from_entropy();
            let x =  rand::Rng::gen::<f32>(&mut random_generator);
            let shard_id = (x * 15f32).floor() as u32;

            let shard_id_option = match shard_id 
            {
                id if id <= 15 => Some(id + 6), // 15 shards with ids starting from 6
                _ => None
            };

            if let Some(shard_id) = shard_id_option 
            {
                let reward = InventoryItem 
                {
                    item_id: shard_id, // this is to use 0 and 1 as soft and hard currency, we need to read definitions...
                    equipped:0,
                    amount: 1,
                };

                attacker.add_inventory_item(reward);

                characters_rewards_summary.push(HeroReward
                {
                    player_id: character_id,
                    item_id: shard_id,
                    amount: 1,
                    inventory_hash: attacker.inventory_version,
                });
            } 

            characters_rewards_summary.push(HeroReward
            {
                player_id: character_id,
                item_id: 2,
                amount: 1,
                inventory_hash: attacker.inventory_version,
            });

            characters_rewards_summary.push(HeroReward
            {
                player_id: character_id,
                item_id: 5,
                amount: xp as u16,
                inventory_hash: attacker.inventory_version,
            });

        }

        let attacker_stored = attacker.clone();
        let defender_stored = defender.clone();

        if let Some(character) = character_entities.get_mut(&character_id)
        {
            *character = attacker;
        }
        if let Some(mob) = mobs.get_mut(&mob_id)
        {
            *mob = defender;
        }
        drop(character_entities);
        drop(mobs);

        //here
        let region_for_positions = map.get_mob_positions_region_from_child(&mob_tile_id);
        let mut mob_positions = region_for_positions.lock().await;

        mob_positions.remove(&mob_tile_id);
        drop(mob_positions);

        attack_details_summary.push(AttackResult
        {
            id: (current_time % 10000) as u16,
            card_id,
            attacker_mob_id: 0,
            attacker_character_id: character_id,
            target_character_id: 0,
            target_mob_id: mob_id,
            target_tile_id: TetrahedronId::default(),
            battle_type: BATTLE_CHAR_MOB,
            result,
        });


        mobs_summary.push(defender_stored.clone());
        characters_summary.push(attacker_stored.clone());

        tx_pe_gameplay_longterm.send(attacker_stored).await.unwrap();
        tx_moe_gameplay_webservice.send(defender_stored).await.unwrap();
    }
}


pub async fn cast_hero_from_mob(
    map : &Arc<GameMap>,
    current_time : u64,
    server_state: &Arc<ServerState>,
    tx_moe_gameplay_webservice : &GaiaSender<MobEntity>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    mobs_summary : &mut Vec<MobEntity>,
    characters_summary : &mut Vec<HeroEntity>,
    attack_details_summary : &mut Vec<AttackResult>,
    card_id: u32,
    hero_id:u16,
    mob_id: u32,
    mob_tile_id: TetrahedronId,
    missed: u8,
)
{
    cli_log::info!("----- attack character ");
    let mut character_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;

    if let Some(defender)= character_entities.get_mut(&hero_id)
    {
        if defender.get_flag_value(INSIDE_TOWER_FLAG)
        {
            // cannot touch someone in the tower
            return;
        }
        else if defender.get_flag_value(TRYING_TO_ENTER_TOWER_FLAG)
        {
            defender.set_flag(TRYING_TO_ENTER_TOWER_FLAG, false);
        }
    }
    let character_defender_option = character_entities.get(&hero_id);

    let mob_region = map.get_mob_region_from_child(&mob_tile_id);
    let mut mobs = mob_region.lock().await;

    let mob_attacker_option = mobs.get(&mob_id);
    
    let current_time_in_seconds = (current_time / 1000) as u32;
    if let (Some(attacker), Some(defender)) = (mob_attacker_option, character_defender_option)
    {
        let mut attacker = attacker.clone();
        let mut defender = defender.clone();
        let result = super::utils::attack::<MobEntity, HeroEntity>(&map.definitions, card_id, current_time_in_seconds, missed, &mut attacker, &mut defender);

        attacker.version += 1;
        defender.version += 1;

        if defender.health <= 0 
        {
            let base_xp = defender.level + 1;
            let factor = 1.1f32.powf((defender.level as i32 - attacker.level as i32).max(0) as f32);
            let xp = base_xp as f32 * factor;
        }


        let attacker_stored = attacker.clone();
        let defender_stored = defender.clone();

        if let Some(character) = character_entities.get_mut(&hero_id)
        {
            *character = defender;
        }
        if let Some(mob) = mobs.get_mut(&mob_id)
        {
            *mob = attacker;
        }
        drop(character_entities);
        drop(mobs);

        attack_details_summary.push(AttackResult
        {
            id: (current_time % 10000) as u16,
            card_id,
            attacker_mob_id: mob_id,
            attacker_character_id: 0,
            target_character_id: hero_id,
            target_mob_id: 0,
            target_tile_id: TetrahedronId::default(),
            battle_type: BATTLE_MOB_CHAR,
            result,
        });

        characters_summary.push(defender_stored.clone());
        mobs_summary.push(attacker_stored.clone());

        tx_pe_gameplay_longterm.send(defender_stored).await.unwrap();
        tx_moe_gameplay_webservice.send(attacker_stored).await.unwrap();
    }
}

pub async fn check_buffs(
    map : &Arc<GameMap>,
    current_time : u64,
    server_state: &Arc<ServerState>,
    tx_moe_gameplay_webservice : &GaiaSender<MobEntity>,
    mobs_summary : &mut Vec<MobEntity>,
    mob_id : u32,
    mob_tile_id: TetrahedronId,
)
{
    let mob_region = map.get_mob_region_from_child(&mob_tile_id);
    let mut mobs = mob_region.lock().await;

    let mob_attacker_option = mobs.get_mut(&mob_id);
    
    let current_time_in_seconds = (current_time / 1000) as u32;
    if let Some(mob) = mob_attacker_option
    {
        cli_log::info!("-----  check buffs for {} {:?}", mob_id, mob.buffs_summary );

        if mob.has_expired_buffs(current_time_in_seconds)
        {
            cli_log::info!("------- check removing expired buffs");
            mob.removed_expired_buffs(current_time_in_seconds);
        }

        mob.version += 1;
        let mob_copy = mob.clone();
        drop(mobs);

        mobs_summary.push(mob_copy.clone());
        tx_moe_gameplay_webservice.send(mob_copy).await.unwrap();
    }
}