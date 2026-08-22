
use rand::rngs::StdRng;

use crate::ability_user::attack_result::BLOCKED_ATTACK_RESULT;
use crate::ability_user::attack_result::MISSED_ATTACK_RESULT;
use crate::ability_user::attack_result::NORMAL_ATTACK_RESULT;
use crate::ability_user::AbilityUser;
use crate::buffs::buff::BuffUser;
use crate::buffs::buff::BUFF_DEFENSE;
use crate::buffs::buff::BUFF_STRENGTH;
use crate::definitions::damage_table;
use crate::hero::hero_command::HeroCommand;
use crate::definitions::definitions_container::Definitions;
use crate::map::map_entity::MapCommand;
use crate::mob::mob_command::MobCommand;
use crate::tower::TowerCommand;

fn short_type_name<X: ?Sized>() -> &'static str
{
    let full_name = std::any::type_name::<X>();
    full_name.rsplit("::").next().unwrap_or(full_name)
}

pub fn attack<T:AbilityUser+BuffUser, S:AbilityUser+BuffUser>(
    definitions : &Definitions,
    card_id:u32,
    current_time_in_seconds: u32,
    missed:u8,
    attacker: &mut T,
    target : &mut S) -> u8
{
    // --------------- just for the record
    let mut summary = String::new();
    let attacker_type = short_type_name::<T>();
    let target_type = short_type_name::<S>();
    summary.push_str(&format!("\n=== Battle Summary: {attacker_type} vs {target_type} (card {card_id}) ===\n"));
    // --------------- just for the record

    // first we roll
    let attack_roll = roll_d100();

    summary.push_str(&format!("-- Attack roll = {attack_roll}\n"));

    // --------------- just for the record
    let (attacker_strength, attacker_vitality, attacker_agility, attacker_will) = attacker.get_stats(definitions);
    summary.push_str(&format!(
        "-- Attacker stats -> strength:{attacker_strength} vitality:{attacker_vitality} agility:{attacker_agility} will:{attacker_will}\n"
    ));
    // --------------- just for the record

    // --------------- just for the record
    summary.push_str("Total attack breakdown:\n");
    // --------------- just for the record

    let offensive_bonus = attacker.get_total_attack_with_summary(card_id, definitions, &mut summary);
    attacker.use_buffs(vec![BUFF_STRENGTH], definitions);

    let skill_bonus = attacker.get_skill_bonus_with_summary(card_id, definitions, &mut summary);

    // --------------- just for the record
    // --------------- just for the record
    // summary.push_str("Strength buff consumed from attacker\n");
    // --------------- just for the record

    if missed == 1
    {
    // --------------- just for the record
        summary.push_str("-- Attack missed, no damage dealt\n=== End Summary ===\n");
        cli_log::info!("{summary}");
    // --------------- just for the record
        return MISSED_ATTACK_RESULT;
    }

    // --------------- just for the record
    let (target_strength, target_vitality, target_agility, target_will) = target.get_stats(definitions);
    summary.push_str(&format!(
        "-- Target stats -> strength:{target_strength} vitality:{target_vitality} agility:{target_agility} will:{target_will}\n"
    ));
    // --------------- just for the record

    let defensive_bonus = target.get_total_defense(definitions);
    // --------------- just for the record
    summary.push_str(&format!("-- Total defense (from agility) = {defensive_bonus}\n"));
    // --------------- just for the record

    target.use_buffs(vec![BUFF_DEFENSE], definitions);
    // --------------- just for the record
    // summary.push_str("6. Defense buff consumed from target\n");
    // --------------- just for the record

    let attack = attack_roll + offensive_bonus as i32 + skill_bonus as i32;
    summary.push_str(&format!("-- Attack total = attack({attack_roll}) + offensive_bonus({offensive_bonus}) + skill bonus ({skill_bonus}) = {attack}\n"));

    let attack_result = (attack - defensive_bonus as i32).max(0) as i16;
    summary.push_str(&format!("-- Attack Result = attack({attack}) - defense({defensive_bonus}) = {attack_result}\n"));

    // now we check the weapon vs armor table.

    let card_data = definitions.cards.get(card_id as usize).unwrap();

    // we need the defender armor type

    let armor_type = target.get_armor_type( definitions);
    summary.push_str(&format!("-- Defender armor type = {armor_type}\n"));

    summary.push_str(&format!("-- Calculating Damage:\n"));
    let damage_result = damage_table::get_damage(&definitions.damage_table, attack_result, &card_data.damage_type, &armor_type).unwrap_or((0, 'A'));

    let damage_value = damage_result.0;
    let critical_type = damage_result.1;
    summary.push_str(&format!("-- Damage Result damage {damage_value} with critical {critical_type}\n"));

    if damage_value <= 0
    {
        summary.push_str("-- Damage is negative or 0, attack blocked\n=== End Summary ===\n");
        cli_log::info!("{summary}");
        return BLOCKED_ATTACK_RESULT;
    }

    let health = target.get_health();
    let updated_health = health.saturating_sub(damage_value as u16);
    summary.push_str(&format!("-- Target health {health} -> {updated_health}\n"));
    target.update_health(updated_health, definitions);

    if health == updated_health
    {
        summary.push_str("-- Health unchanged, attack blocked\n=== End Summary ===\n");
        cli_log::info!("{summary}");
        return BLOCKED_ATTACK_RESULT;
    }
    else
    {
        if let Some(skill) = definitions.cards.get(card_id as usize)
        {
            let mut random_generator = <StdRng as rand::SeedableRng>::from_entropy();
            let x =  rand::Rng::gen::<f32>(&mut random_generator);
            summary.push_str(&format!("-- Skill effect roll {x:.2} vs probability {}\n", skill.effect_probability));
            if x <= skill.effect_probability
            {
                if let Some(skill_def) = definitions.buffs.get(&skill.buff)
                {
                    target.add_buff(skill_def.code, current_time_in_seconds + 10, definitions);
                    summary.push_str(&format!("-- Buff '{}' applied to target, expires at {}\n", skill_def.code, current_time_in_seconds + 10));
                }
            }
        }

        summary.push_str("=== End Summary ===\n");
        cli_log::info!("{summary}");
        return NORMAL_ATTACK_RESULT;
    }
}

pub fn roll_d100() -> i32
{
    let mut random_generator = <StdRng as rand::SeedableRng>::from_entropy();
    let first_roll = rand::Rng::gen_range(&mut random_generator, 1..=100u32) as i32;
    let mut total = first_roll;

    if first_roll >= 96
    {
        loop
        {
            let roll = rand::Rng::gen_range(&mut random_generator, 1..=100u32) as i32;
            total += roll;

            if roll < 96
            {
                break;
            }
        }
    }
    else if first_roll <= 5
    {
        loop
        {
            let roll = rand::Rng::gen_range(&mut random_generator, 1..=100u32) as i32;
            total -= roll;

            if roll > 5
            {
                break;
            }
        }
    }

    total
}

pub fn heal<T:AbilityUser+BuffUser, S:AbilityUser+BuffUser>(
    definitions : &Definitions,
    _card_id:u32,
    _current_time_in_seconds: u32,
    _caster: &mut T,
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

//     cli_log::info!("reward {:?}", reward);

//     players_rewards_summary.push(reward);
//     players_summary.push(player_entity.clone());
// }


pub fn get_tile_commands_to_execute(current_time : u64, delayed_tile_commands_guard : &mut Vec<(u64, MapCommand)>) -> Vec<MapCommand>
{
    let mut items_to_execute = Vec::<MapCommand>::new();
    // let current_time = time.load(std::sync::atomic::Ordering::Relaxed);

    delayed_tile_commands_guard.retain(|b| 
    {
        let should_execute = b.0 <= current_time;
        // cli_log::info!("checking delayed action {} task_time {} current_time {current_time}", should_execute, b.0);
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
        // cli_log::info!("checking delayed action {} task_time {} current_time {current_time}", should_execute, b.0);
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
        // cli_log::info!("checking delayed action {} task_time {} current_time {current_time}", should_execute, b.0);
        if should_execute
        {
            items_to_execute.push(b.1.clone());
        }

        !should_execute // we keep items that we didn't execute
    });

    items_to_execute
}


pub fn get_player_commands_to_execute(current_time : u64, delayed_player_commands_guards : &mut Vec<(u64, HeroCommand)>) -> Vec<HeroCommand>
{
    let mut player_commands_to_execute = Vec::<HeroCommand>::new();

    // cli_log::info!("checking delayed plaeyr commands {}" , delayed_commands_lock.len());
    delayed_player_commands_guards.retain(|b| 
    {
        let should_execute = b.0 <= current_time;
        // cli_log::info!("checking delayed player action {} task_time {} current_time {current_time}", should_execute, b.0);
        if should_execute
        {
            player_commands_to_execute.push(b.1.clone());
        }

        !should_execute // we keep items that we didn't execute
    });

    player_commands_to_execute
}
