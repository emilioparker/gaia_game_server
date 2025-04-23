use std::{collections::HashMap, sync::Arc, u16};
use rand::rngs::StdRng;
use tokio::sync::{mpsc::Sender, Mutex};
use crate::{ability_user::{attack::Attack, attack_result::{AttackResult, BATTLE_CHAR_MOB, BATTLE_MOB_CHAR, BATTLE_MOB_MOB}}, buffs::buff::BuffUser, hero::hero_inventory::InventoryItem, definitions::definitions_container::Definitions, gaia_mpsc::GaiaSender, map::{tetrahedron_id::TetrahedronId, GameMap}, mob::{mob_command::{self, MobCommand}, mob_instance::MobEntity}, ServerState};
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
            match &mobs_command.info 
            {
                mob_command::MobCommandInfo::Touch() => 
                {
                    check_buffs(
                        &map,
                        current_time,
                        &server_state,
                        tx_moe_gameplay_webservice,
                        mobs_summary, 
                        mobs_command.tile_id.clone()).await;
                },
                mob_command::MobCommandInfo::CastFromMobToMob(caster_mob_tile_id,card_id,required_time,active_effect,missed) => 
                {
                    let end_time = current_time + *required_time as u64;
                    if *required_time == 0
                    {
                        cast_mob_from_mob(
                            &map,
                            current_time,
                            &server_state,
                            tx_moe_gameplay_webservice,
                            mobs_summary,
                            attack_details_summary,
                            *card_id,
                            caster_mob_tile_id.clone(),
                            mobs_command.tile_id.clone(),
                            *missed,
                        ).await;
                    }
                    else
                    {
                        cli_log::info!("------------ required time for cast to mob {required_time} current time: {current_time} {card_id}");
                        let mut lock = delayed_mob_commands_lock.lock().await;
                        let info = mob_command::MobCommandInfo::CastFromMobToMob(caster_mob_tile_id.clone(), *card_id, *required_time, *active_effect, *missed);
                        let mob_action = MobCommand { tile_id : mobs_command.tile_id.clone(), info };
                        lock.push((end_time, mob_action));
                        drop(lock);

                        // we only send attack messages if attack is delayed, for projectiles and other instances.
                        let attack = Attack
                        {
                            id: (current_time % 10000) as u16,
                            attacker_character_id: 0,
                            target_character_id: 0,
                            attacker_mob_tile_id: caster_mob_tile_id.clone(),
                            target_mob_tile_id: mobs_command.tile_id.clone(),
                            card_id: *card_id,
                            required_time: *required_time,
                            active_effect: *active_effect,
                            battle_type : BATTLE_MOB_MOB,
                        };

                        cli_log::info!("--- cast {} effect {}", attack.required_time, attack.active_effect);
                        attacks_summary.push(attack);
                    }

                },
                mob_command::MobCommandInfo::CastFromCharacterToMob(character_id, card_id, required_time, active_effect, missed) => 
                {
                    let end_time = current_time + *required_time as u64;
                    if *required_time == 0
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
                            *card_id,
                            *character_id,
                            mobs_command.tile_id.clone(),
                            *missed,
                        ).await;
                    }
                    else 
                    {
                        cli_log::info!("------------ required time for attack to mob {required_time} current time: {current_time} {card_id}");
                        let mut lock = delayed_mob_commands_lock.lock().await;
                        let info = mob_command::MobCommandInfo::CastFromCharacterToMob(*character_id, *card_id, *required_time, *active_effect, *missed);
                        let mob_action = MobCommand { tile_id : mobs_command.tile_id.clone(), info };
                        lock.push((end_time, mob_action));
                        drop(lock);

                        // we only send attack messages if attack is delayed, for projectiles and other instances.
                        let attack = Attack
                        {
                            id: (current_time % 10000) as u16,
                            attacker_character_id: *character_id,
                            target_character_id: 0,
                            attacker_mob_tile_id: TetrahedronId::default(),
                            target_mob_tile_id: mobs_command.tile_id.clone(),
                            card_id: *card_id,
                            required_time: *required_time,
                            active_effect: *active_effect,
                            battle_type : BATTLE_CHAR_MOB,
                        };

                        cli_log::info!("--- attack {} effect {}", attack.required_time, attack.active_effect);
                        attacks_summary.push(attack);
                    }

                },
                mob_command::MobCommandInfo::Spawn(character_id, mob_id, level) => 
                {
                    spawn_mob(&map, &server_state, tx_moe_gameplay_webservice, mobs_summary, mobs_command.tile_id.clone(), current_time, *character_id, *mob_id, *level).await;
                },
                mob_command::MobCommandInfo::ControlMob(character_id) => 
                {
                    control_mob(&map, &server_state, tx_moe_gameplay_webservice, mobs_summary, mobs_command.tile_id.clone(), current_time, *character_id).await;
                },
                mob_command::MobCommandInfo::AttackFromMobToWalker(character_id, card_id, required_time, active_effect, missed) => 
                {
                    let end_time = current_time + *required_time as u64;
                    if *required_time == 0
                    {
                        cast_character_from_mob(
                            &map,
                            current_time,
                            &server_state,
                            tx_moe_gameplay_webservice,
                            tx_pe_gameplay_longterm,
                            mobs_summary,
                            characters_summary,
                            attack_details_summary,
                            *card_id,
                            *character_id,
                            mobs_command.tile_id.clone(),
                            *missed,
                        ).await;
                    }
                    else 
                    {
                        cli_log::info!("------------ required time for attack to character from mob {required_time} current time: {current_time} {card_id}");
                        let mut lock = delayed_mob_commands_lock.lock().await;
                        let info = mob_command::MobCommandInfo::AttackFromMobToWalker(*character_id, *card_id, *required_time, *active_effect, *missed);
                        let mob_action = MobCommand { tile_id : mobs_command.tile_id.clone(), info };
                        lock.push((end_time, mob_action));
                        drop(lock);

                        // we only send attack messages if attack is delayed, for projectiles and other instances.
                        let attack = Attack
                        {
                            id: (current_time % 10000) as u16,
                            attacker_character_id: 0,
                            target_character_id: *character_id,
                            attacker_mob_tile_id: mobs_command.tile_id.clone(),
                            target_mob_tile_id: TetrahedronId::default(),
                            card_id: *card_id,
                            required_time: *required_time,
                            active_effect: *active_effect,
                            battle_type : BATTLE_MOB_CHAR,
                        };

                        cli_log::info!("--- attack {} effect {}", attack.required_time, attack.active_effect);
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
        match &mobs_command.info 
        {
            mob_command::MobCommandInfo::Touch() => todo!(),
            mob_command::MobCommandInfo::Spawn(_, _, _) => todo!(),
            mob_command::MobCommandInfo::ControlMob(_) => todo!(),
            mob_command::MobCommandInfo::CastFromMobToMob(caster_mob_tile_id,card_id,required_time,active_effect,missed) => 
            {
                cast_mob_from_mob(
                    &map,
                    current_time,
                    &server_state,
                    tx_moe_gameplay_webservice,
                    mobs_summary,
                    attack_details_summary,
                    *card_id,
                    caster_mob_tile_id.clone(),
                    mobs_command.tile_id.clone(),
                    *missed,
                ).await;
            }
            mob_command::MobCommandInfo::CastFromCharacterToMob(character_id, card_id, _required_time, _active_effect, missed) => 
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
                    *card_id,
                    *character_id,
                    mobs_command.tile_id.clone(),
                    *missed,
                ).await;
            },
            mob_command::MobCommandInfo::AttackFromMobToWalker(character_id, card_id, _requirec_time, _active_effect, missed) => 
            {
                cast_character_from_mob(
                    &map,
                    current_time,
                    &server_state,
                    tx_moe_gameplay_webservice,
                    tx_pe_gameplay_longterm,
                    mobs_summary,
                    characters_summary,
                    attack_details_summary,
                    *card_id,
                    *character_id,
                    mobs_command.tile_id.clone(),
                    *missed,
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
    mob_id: u32,
    level: u8
)
{
    let current_time_in_seconds = (current_time / 1000) as u32;
    let mut new_mob = MobEntity
    {
        tile_id : tile_id.clone(),
        mob_definition_id: mob_id as u16,
        level,
        version: 0,
        owner_id: character_id,
        ownership_time: current_time_in_seconds,
        origin_id: tile_id.clone(),
        target_id: tile_id.clone(),
        time: 0,
        health: 0,
        buffs: Vec::new(),
        buffs_summary: [0,0,0,0,0],
    };


    if let Some(mob_progression) = map.definitions.mob_progression_by_mob.get(mob_id as usize)
    {
        if let Some(entry) = mob_progression.get(level as usize) 
        {
            new_mob.health =  entry.constitution;
        }
    }

    let region = map.get_mob_region_from_child(&tile_id);
    let mut mobs = region.lock().await;
    if let Some(mob) = mobs.get_mut(&tile_id)
    {
        if mob.health <= 0 // we can spawn a mob here.
        {
            new_mob.version = mob.version + 1;
            // cli_log::info!("new mob {:?}", updated_tile);
            *mob = new_mob.clone();
            drop(mobs);
            mobs_summary.push(new_mob.clone());
            tx_moe_gameplay_webservice.send(new_mob).await.unwrap();
        }
        else 
        {
            mobs_summary.push(mob.clone());
            drop(mobs);
        }
    }
    else
    {
        mobs_summary.push(new_mob.clone());
        mobs.insert(tile_id, new_mob.clone());
        drop(mobs);

        tx_moe_gameplay_webservice.send(new_mob).await.unwrap();
    }
}

pub async fn control_mob(
    map : &Arc<GameMap>,
    server_state: &Arc<ServerState>,
    tx_moe_gameplay_webservice : &GaiaSender<MobEntity>,
    mobs_summary : &mut Vec<MobEntity>,
    tile_id: TetrahedronId,
    current_time : u64,
    player_id: u16
)
{
    let mob_region = map.get_mob_region_from_child(&tile_id);
    let mut mobs = mob_region.lock().await;
    if let Some(mob) = mobs.get_mut(&tile_id)
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
pub async fn cast_mob_from_mob(
    map : &Arc<GameMap>,
    current_time : u64,
    server_state: &Arc<ServerState>,
    tx_moe_gameplay_webservice : &GaiaSender<MobEntity>,
    mobs_summary : &mut Vec<MobEntity>,
    attack_details_summary : &mut Vec<AttackResult>,
    card_id: u32,
    caster_mob_id: TetrahedronId,
    target_mob_id: TetrahedronId,
    missed: u8,
)
{
    cli_log::info!("----- cast to mob from mob ");

    // let key = self.get_parent(tetrahedron_id);

    let mob_region = map.get_mob_region_from_child(&caster_mob_id);
    let mut mobs = mob_region.lock().await;
    let mob_caster_option = mobs.get(&caster_mob_id);
    let mob_target_option = mobs.get(&target_mob_id);

    let current_time_in_seconds = (current_time / 1000) as u32;
    if let (Some(mob_caster), Some(mob_target)) = (mob_caster_option, mob_target_option)
    {
        let mut caster = mob_caster.clone();
        let mut target = mob_target.clone();
        cli_log::info!("---- calling heal {} id: {}" , mob_target.health, mob_target.tile_id);
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
            attacker_mob_tile_id: caster_mob_id,
            attacker_character_id: 0,
            target_character_id: 0,
            target_mob_tile_id: target_mob_id,
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
    mob_id: TetrahedronId,
    missed: u8,
)
{
    cli_log::info!("----- attack mob ");
    let mut character_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
    let character_attacker_option = character_entities.get(&character_id);

    let mob_region = map.get_mob_region_from_child(&mob_id);
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

        attack_details_summary.push(AttackResult
        {
            id: (current_time % 10000) as u16,
            card_id,
            attacker_mob_tile_id: TetrahedronId::default(),
            attacker_character_id: character_id,
            target_character_id: 0,
            target_mob_tile_id: mob_id,
            battle_type: BATTLE_CHAR_MOB,
            result,
        });


        mobs_summary.push(defender_stored.clone());
        characters_summary.push(attacker_stored.clone());

        tx_pe_gameplay_longterm.send(attacker_stored).await.unwrap();
        tx_moe_gameplay_webservice.send(defender_stored).await.unwrap();
    }
}


pub async fn cast_character_from_mob(
    map : &Arc<GameMap>,
    current_time : u64,
    server_state: &Arc<ServerState>,
    tx_moe_gameplay_webservice : &GaiaSender<MobEntity>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    mobs_summary : &mut Vec<MobEntity>,
    characters_summary : &mut Vec<HeroEntity>,
    attack_details_summary : &mut Vec<AttackResult>,
    card_id: u32,
    character_id:u16,
    mob_id: TetrahedronId,
    missed: u8,
)
{
    cli_log::info!("----- attack character ");
    let mut character_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
    let character_defender_option = character_entities.get(&character_id);

    let mob_region = map.get_mob_region_from_child(&mob_id);
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

        if let Some(character) = character_entities.get_mut(&character_id)
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
            attacker_mob_tile_id: mob_id,
            attacker_character_id: 0,
            target_character_id: character_id,
            target_mob_tile_id: TetrahedronId::default(),
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
    mob_id: TetrahedronId,
)
{
    let mob_region = map.get_mob_region_from_child(&mob_id);
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