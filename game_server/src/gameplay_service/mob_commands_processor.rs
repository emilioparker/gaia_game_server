use std::{collections::HashMap, sync::Arc, u16};
use tokio::sync::{mpsc::Sender, Mutex};
use crate::{ability_user::{attack::Attack, attack_result::{AttackResult, BATTLE_CHAR_MOB, BATTLE_MOB_CHAR}}, buffs::buff::{BuffUser, Stat}, character::character_entity::InventoryItem, definitions::definitions_container::Definitions, map::{tetrahedron_id::TetrahedronId, GameMap}, mob::{mob_command::{self, MobCommand}, mob_instance::MobEntity}, ServerState};
use crate::character::{character_entity::CharacterEntity, character_reward::CharacterReward};

pub async fn process_mob_commands (
    map : Arc<GameMap>,
    current_time : u64,
    server_state: Arc<ServerState>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    tx_moe_gameplay_webservice : &Sender<MobEntity>,
    mobs_commands_processor_lock : Arc<Mutex<Vec<MobCommand>>>,
    delayed_mob_commands_lock : Arc<Mutex<Vec<(u64, MobCommand)>>>,
    mobs_summary : &mut Vec<MobEntity>,
    characters_summary : &mut  Vec<CharacterEntity>,
    attack_details_summary : &mut Vec<AttackResult>,
    rewards_summary : &mut  Vec<CharacterReward>,
    player_attacks_summary : &mut  Vec<Attack>,
)
{
    let mut mobs_commands_data = mobs_commands_processor_lock.lock().await;

    // println!("mobs commands len {}", mobs_commands_data.len());
    if mobs_commands_data.len() > 0 
    {
        for mobs_command in mobs_commands_data.iter()
        {
            match &mobs_command.info 
            {
                mob_command::MobCommandInfo::Touch() => 
                {

                },
                mob_command::MobCommandInfo::Attack(character_id, card_id, required_time, active_effect, missed) => 
                {
                    let end_time = current_time + *required_time as u64;
                    if *required_time == 0
                    {
                        attack_mob(
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
                        println!("------------ required time for attack to mob {required_time} current time: {current_time} {card_id}");
                        let mut lock = delayed_mob_commands_lock.lock().await;
                        let info = mob_command::MobCommandInfo::Attack(*character_id, *card_id, *required_time, *active_effect, *missed);
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

                        println!("--- attack {} effect {}", attack.required_time, attack.active_effect);
                        player_attacks_summary.push(attack);
                    }

                },
                mob_command::MobCommandInfo::Spawn(character_id, mob_id, level) => 
                {
                    spawn_mob(&map, &server_state, tx_moe_gameplay_webservice, mobs_summary, mobs_command.tile_id.clone(), current_time, *character_id, *mob_id, *level).await;
                },
                mob_command::MobCommandInfo::ControlMapEntity(character_id) => 
                {
                    control_mob(&map, &server_state, tx_moe_gameplay_webservice, mobs_summary, mobs_command.tile_id.clone(), current_time, *character_id).await;
                },
                mob_command::MobCommandInfo::AttackWalker(character_id, card_id, required_time, active_effect, missed) => 
                {
                    let end_time = current_time + *required_time as u64;
                    if *required_time == 0
                    {
                        attack_character_from_mob(
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
                        println!("------------ required time for attack to character from mob {required_time} current time: {current_time} {card_id}");
                        let mut lock = delayed_mob_commands_lock.lock().await;
                        let info = mob_command::MobCommandInfo::Attack(*character_id, *card_id, *required_time, *active_effect, *missed);
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

                        println!("--- attack {} effect {}", attack.required_time, attack.active_effect);
                        player_attacks_summary.push(attack);
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
    tx_moe_gameplay_webservice : &Sender<MobEntity>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    mobs_summary : &mut Vec<MobEntity>,
    characters_summary : &mut Vec<CharacterEntity>,
    attack_details_summary : &mut Vec<AttackResult>,
    rewards_summary : &mut Vec<CharacterReward>,
    delayed_mob_commands_to_execute : Vec<MobCommand>
)
{
    for mobs_command in delayed_mob_commands_to_execute.iter()
    {
        match &mobs_command.info 
        {
            mob_command::MobCommandInfo::Touch() => todo!(),
            mob_command::MobCommandInfo::Spawn(_, _, _) => todo!(),
            mob_command::MobCommandInfo::ControlMapEntity(_) => todo!(),
            mob_command::MobCommandInfo::Attack(character_id, card_id, _required_time, _active_effect, missed) => 
            {
                attack_mob(
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
            mob_command::MobCommandInfo::AttackWalker(character_id, card_id, _requirec_time, _active_effect, missed) => 
            {
                attack_character_from_mob(
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
    tx_moe_gameplay_webservice : &Sender<MobEntity>,
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
        buffs_summary: [(0,0),(0,0),(0,0),(0,0),(0,0)],
    };


    if let Some(entry) = map.definitions.mob_progression.get(level as usize) 
    {
        // let attribute = (entry.skill_points / 4) as u16;
        new_mob.health =  entry.constitution as i32;
        // updated_mob.strength = attribute; // attack
        // updated_mob.dexterity = attribute; // attack
    }

    let region = map.get_mob_region_from_child(&tile_id);
    let mut mobs = region.lock().await;
    if let Some(mob) = mobs.get_mut(&tile_id)
    {
        if mob.health <= 0 // we can spawn a mob here.
        {
            new_mob.version = mob.version + 1;
            // println!("new mob {:?}", updated_tile);
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
    tx_moe_gameplay_webservice : &Sender<MobEntity>,
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
        // println!("for mob {id} time {current_time_in_seconds} tile time: {tile_time} difference :{difference}");

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
            // println!("updating time {current_time} {}", updated_tile.ownership_time);
            updated_mob.version += 1;
            updated_mob.owner_id = player_id;
            updated_mob.ownership_time = current_time_in_seconds; // seconds of control
            // println!("new time {}", updated_tile.ownership_time);
        }

        mobs_summary.push(updated_mob.clone());
        *mob = updated_mob.clone();
        drop(mobs);

        let capacity = tx_moe_gameplay_webservice.capacity();
        server_state.tx_moe_gameplay_webservice.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);

        // sending the updated tile somewhere.
        tx_moe_gameplay_webservice.send(updated_mob.clone()).await.unwrap();
    }
}

pub async fn attack_mob(
    map : &Arc<GameMap>,
    current_time : u64,
    server_state: &Arc<ServerState>,
    tx_moe_gameplay_webservice : &Sender<MobEntity>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    mobs_summary : &mut Vec<MobEntity>,
    characters_summary : &mut Vec<CharacterEntity>,
    attack_details_summary : &mut Vec<AttackResult>,
    characters_rewards_summary : &mut Vec<CharacterReward>,
    card_id: u32,
    character_id:u16,
    mob_id: TetrahedronId,
    missed: u8,
)
{
    println!("----- attack mob ");
    let mut character_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.character.lock().await;
    let character_defender_option = character_entities.get(&character_id);

    let mob_region = map.get_mob_region_from_child(&mob_id);
    let mut mobs = mob_region.lock().await;

    let mob_attacker_option = mobs.get(&mob_id);
    
    if let (Some(attacker), Some(defender)) = (mob_attacker_option, character_defender_option)
    {
        let mut attacker = attacker.clone();
        let mut defender = defender.clone();
        let result = super::utils::attack::<MobEntity, CharacterEntity>(&map.definitions, card_id, missed, &mut attacker, &mut defender);

        attacker.version += 1;
        defender.version += 1;


        if defender.health <= 0 
        {
            let base_xp = defender.level + 1;
            let factor = 1.1f32.powf((defender.level as i32 - attacker.level as i32).max(0) as f32);
            let xp = base_xp as f32 * factor;

            println!("base_xp:{base_xp} - factor:{factor} xp: {xp}");
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

        // metrics
        let capacity = tx_moe_gameplay_webservice.capacity();
        server_state.tx_moe_gameplay_webservice.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);

        let capacity = tx_pe_gameplay_longterm.capacity();
        server_state.tx_pe_gameplay_longterm.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
    }
}


pub async fn attack_character_from_mob(
    map : &Arc<GameMap>,
    current_time : u64,
    server_state: &Arc<ServerState>,
    tx_moe_gameplay_webservice : &Sender<MobEntity>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    mobs_summary : &mut Vec<MobEntity>,
    characters_summary : &mut Vec<CharacterEntity>,
    attack_details_summary : &mut Vec<AttackResult>,
    card_id: u32,
    character_id:u16,
    mob_id: TetrahedronId,
    missed: u8,
)
{
    println!("----- attack character ");
    let mut character_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.character.lock().await;
    let character_defender_option = character_entities.get(&character_id);

    let mob_region = map.get_mob_region_from_child(&mob_id);
    let mut mobs = mob_region.lock().await;

    let mob_attacker_option = mobs.get(&mob_id);
    
    if let (Some(attacker), Some(defender)) = (mob_attacker_option, character_defender_option)
    {
        let mut attacker = attacker.clone();
        let mut defender = defender.clone();
        let result = super::utils::attack::<MobEntity, CharacterEntity>(&map.definitions, card_id, missed, &mut attacker, &mut defender);

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

        // metrics
        let capacity = tx_moe_gameplay_webservice.capacity();
        server_state.tx_moe_gameplay_webservice.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);

        let capacity = tx_pe_gameplay_longterm.capacity();
        server_state.tx_pe_gameplay_longterm.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
    }
}