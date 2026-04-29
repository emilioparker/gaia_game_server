use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use crate::ability_user::attack::Attack;
use crate::ability_user::attack_result::AttackResult;
use crate::ability_user::attack_result::BATTLE_CHAR_CHAR;
use crate::definitions::items::ItemUsage;
use crate::gaia_mpsc::GaiaSender;
use crate::hero::hero_card_inventory::CardItem;
use crate::hero::hero_command;
use crate::hero::hero_command::HeroCommand;
use crate::hero::hero_command::HeroCommandInfo;
use crate::hero::hero_entity::HeroEntity;
use crate::hero::hero_entity::CHAT_FLAG;
use crate::hero::hero_equipment_inventory::EquipmentItem;
use crate::hero::hero_inventory::InventoryItem;
use crate::hero::hero_presentation::HeroPresentation;
use crate::hero::hero_reward::HeroReward;
use crate::map::tetrahedron_id::TetrahedronId;
use crate::map::GameMap;
use crate::ServerState;
use crate::buffs::buff::BuffUser;

pub async fn process_hero_commands (
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    current_time : u64,
    hero_commands_processor_lock : Arc<Mutex<Vec<HeroCommand>>>,
    tx_he_gameplay_longterm : &GaiaSender<HeroEntity>,
    heros_summary : &mut Vec<HeroEntity>,
    heros_presentation_summary : &mut Vec<HeroPresentation>,
    attacks_summary : &mut  Vec<Attack>,
    attack_details_summary : &mut  Vec<AttackResult>,
    rewards_summary : &mut Vec<HeroReward>,
    delayed_hero_commands_lock : Arc<Mutex<Vec<(u64, HeroCommand)>>>
)
{
    let mut hero_commands_data = hero_commands_processor_lock.lock().await;

    if hero_commands_data.len() == 0
    {
        return;
    }

    for hero_command in hero_commands_data.iter()
    {
        let cloned_data = hero_command.to_owned();
        if let Some(atomic_time) = map.active_players.get(&cloned_data.player_id)
        {
            atomic_time.store(current_time, std::sync::atomic::Ordering::Relaxed);
        }

        match &hero_command.info 
        {
            hero_command::HeroCommandInfo::Touch() => 
                    {
                        cli_log::error!("touch not implemented")
                    },
            hero_command::HeroCommandInfo::Movement(movement_data) => 
                    {
                        move_character(
                            &map,
                            tx_he_gameplay_longterm,
                            heros_summary,
                            current_time,
                            cloned_data.player_id,
                            movement_data.position.clone(),
                            movement_data.second_position.clone(),
                            movement_data.vertex_id,
                            movement_data.path,
                        ).await;
                    },
            hero_command::HeroCommandInfo::SellItem(_faction, item_id, item_definition_id, inventory_type, amount) => 
                    {
                        sell_item(&map, tx_he_gameplay_longterm, heros_summary, *item_id, *item_definition_id, *inventory_type, cloned_data.player_id, *amount).await
                    },
            hero_command::HeroCommandInfo::BuyItem(_faction, item_id, item_type, amount) => 
                    {
                        buy_item(&map, tx_he_gameplay_longterm, heros_summary, *item_id, *item_type, cloned_data.player_id, *amount).await
                    },
            hero_command::HeroCommandInfo::UseItem(_faction, item_id, amount) => 
                    {
                        use_item(&map, tx_he_gameplay_longterm, heros_summary, *item_id, cloned_data.player_id, *amount).await;
                    },
            hero_command::HeroCommandInfo::EquipItem(equip_data) => 
                    {
                        equip_item(&map, tx_he_gameplay_longterm, heros_summary, equip_data.item_id, equip_data.inventory_type, cloned_data.player_id, equip_data.current_slot,equip_data.new_slot).await;
                    },
            hero_command::HeroCommandInfo::Respawn(respawn_tile) => 
                    {
                        respawn(&map, tx_he_gameplay_longterm, heros_summary, cloned_data.player_id, respawn_tile.clone()).await;
                    },
            hero_command::HeroCommandInfo::Action(action) => 
                    {
                        set_action(&map, current_time, tx_he_gameplay_longterm, heros_summary, cloned_data.player_id, *action).await;
                    },
            hero_command::HeroCommandInfo::Greet() => 
                    {
                        greet(&map, heros_presentation_summary, cloned_data.player_id).await;
                    },
            hero_command::HeroCommandInfo::ActivateBuff(card_id) =>
                    {
                        activate_buff(&map, current_time, tx_he_gameplay_longterm, heros_summary, *card_id, cloned_data.player_id).await;
                    },
            hero_command::HeroCommandInfo::IncreaseSkillRank(skill_id) =>
                    {
                        increase_skill_rank(&map, tx_he_gameplay_longterm, heros_summary, cloned_data.player_id, *skill_id).await;
                    },
            hero_command::HeroCommandInfo::AttackCharacter(other_player_id, card_id, required_time, active_effect, missed) => 
                    {
                        let end_time = current_time + *required_time as u64;
                        if *required_time == 0
                        {
                            attack_character(
                                &map,
                                current_time,
                                &server_state,
                                tx_he_gameplay_longterm,
                                heros_summary,
                                attack_details_summary,
                                rewards_summary,
                                *card_id,
                                cloned_data.player_id,
                                *other_player_id,
                                *missed).await;
                        }
                        else 
                        {
                            cli_log::info!("------------ required time for hero attack {required_time} current time: {current_time} {card_id}");
                            let mut lock = delayed_hero_commands_lock.lock().await;
                            let info = HeroCommandInfo::AttackCharacter(*other_player_id, *card_id, *required_time, *active_effect, *missed);
                            let character_action = HeroCommand { player_id: cloned_data.player_id, info };
                            lock.push((end_time, character_action));
                            drop(lock);

                            let attack = Attack
                            {
                                id: (current_time % 10000) as u16,
                                attacker_hero_id: cloned_data.player_id,
                                target_hero_id: *other_player_id,
                                target_mob_id: 0,
                                attacker_mob_id: 0,
                                card_id: *card_id,
                                target_tile_id: TetrahedronId::default(),
                                required_time: *required_time,
                                battle_type: BATTLE_CHAR_CHAR,
                            };

                            cli_log::info!("--- attack hero {} at {} effect", other_player_id, attack.required_time);
                            attacks_summary.push(attack);
                        }

                    },
            hero_command::HeroCommandInfo::Disconnect() => 
                    {
                        disconnect(&map, tx_he_gameplay_longterm, heros_summary, cloned_data.player_id).await;
                    },
        }
    }
    hero_commands_data.clear();
}


pub async fn process_delayed_hero_commands(
    map : Arc<GameMap>,
    current_time : u64,
    server_state: Arc<ServerState>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    characters_summary : &mut Vec<HeroEntity>,
    attack_details_summary : &mut Vec<AttackResult>,
    rewards_summary : &mut Vec<HeroReward>,
    delayed_character_commands_to_execute : Vec<HeroCommand>,
)
{
    if delayed_character_commands_to_execute.len() == 0
    {
        return;
    }

    for hero_command in delayed_character_commands_to_execute.iter()
    {
        match &hero_command.info 
        {
            hero_command::HeroCommandInfo::AttackCharacter(other_character_id, card_id, _required_time, _active_effect, missed) => 
            {
                attack_character(
                    &map,
                    current_time,
                    &server_state,
                    tx_pe_gameplay_longterm,
                    characters_summary, 
                    attack_details_summary,
                    rewards_summary,
                    *card_id,
                    hero_command.player_id,
                    *other_character_id,
                    *missed).await;
            },
            _ => 
            {
                cli_log::info!("delayed command not supported");
            }
        }
    }
}

pub async fn use_item(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    heros_summary : &mut Vec<HeroEntity>,
    item_id : u32,
    player_id: u16,
    amount: u16)
{
    let item_definition = map.definitions.items.get(item_id as usize);

    let mut hero_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
    let hero_option = hero_entities.get_mut(&player_id);

    match (hero_option, item_definition) 
    {
        (Some(hero_entity), Some(definition)) => 
        {
            let character_definition = map.definitions.character_progression.get(hero_entity.level as usize).unwrap();
            if definition.usage != 0
            {
                let result = hero_entity.remove_inventory_item(InventoryItem
                {
                    item_id,
                    equipped: 0,
                    amount,
                });// remove soft currency

                cli_log::info!("using item with result {} and  {:?}",result, definition.usage);

                match (result, definition.usage)
                {
                    (true, usage) if usage == ItemUsage::Heal as u8 =>  // heal
                    {
                        hero_entity.health = u32::min(character_definition.constitution as u32, hero_entity.health as u32 + 5) as u16;
                        hero_entity.version += 1;
                    },
                    (true, usage) if usage == ItemUsage::AddXp as u8 =>  // heal
                    {
                        hero_entity.available_skill_points += 2;
                        hero_entity.version += 1;
                    },
                    _ => 
                    {
                        cli_log::info!("item {} cannot be used ", item_id);
                    }
                }
            }

            // cli_log::info!("Add health {:?}", hero_entity);
            tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
            heros_summary.push(hero_entity.clone_for_sending());
        },
        _ => 
        {
            cli_log::info!("error buying item");
        }
    }
}

pub async fn equip_item(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    heros_summary : &mut Vec<HeroEntity>,
    item_id : u32,
    inventory_type : u8,
    player_id: u16,
    current_slot: u8,
    new_slot:u8)
{
    let mut hero_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
    let hero_option = hero_entities.get_mut(&player_id);

    match hero_option 
    {
        Some(hero_entity) => 
        {
            if inventory_type == 0
            {
                let result = hero_entity.equip_inventory_item(item_id, current_slot, new_slot);
                cli_log::info!("equip item with result {}",result);

                tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
                heros_summary.push(hero_entity.clone_for_sending());
            }
            else if inventory_type == 1
            {
                let result = hero_entity.equip_card(item_id, current_slot, new_slot);
                cli_log::info!("equip item with result {}",result);

                tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
                heros_summary.push(hero_entity.clone_for_sending());
            }
            else if inventory_type == 2
            {
                let result = hero_entity.equip_equipment(item_id, current_slot, new_slot);
                cli_log::info!("equip equipment with result {}",result);

                tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
                heros_summary.push(hero_entity.clone_for_sending());
            }
        },
        _ => 
        {
            cli_log::info!("error equipping item");
        }
    }
}

pub async fn buy_item(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    heros_summary : &mut Vec<HeroEntity>,
    item_definition_id : u32,
    inventory_type: u8,
    player_id: u16,
    amount: u16)
{
    let mut hero_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
    cli_log::info!("Buy item with id {item_definition_id}, item_type: {inventory_type}");

    let hero_option = hero_entities.get_mut(&player_id);

    if inventory_type == 0
    {
        let cost  = map.definitions.items.get(item_definition_id as usize).map(|d| d.cost);
        cli_log::info!("cost {cost:?}");
        match (hero_option, cost) 
        {
            (Some(hero_entity), Some(cost)) => 
            {
                let result = hero_entity.remove_inventory_item(InventoryItem
                {
                    item_id : 0,
                    equipped : 0,
                    amount : cost * amount,
                });// remove soft currency

                if result || cost == 0
                {
                    hero_entity.add_inventory_item(InventoryItem
                    {
                        item_id: item_definition_id,
                        equipped : 0,
                        amount
                    });// add item currency
                }

                tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
                heros_summary.push(hero_entity.clone_for_sending());
            },
            _ => 
            {
                cli_log::info!("error buying item");
            }
        }
    }
    else if inventory_type == 1
    {
        let cost  = map.definitions.cards.get(item_definition_id as usize).map(|d| d.store_cost);
        cli_log::info!("card cost {cost:?}");
        match (hero_option, cost) 
        {
            (Some(hero_entity), Some(cost)) => 
            {
                let result = hero_entity.remove_inventory_item(InventoryItem
                {
                    item_id : 0,
                    equipped : 0,
                    amount : cost * amount,
                });// remove soft currency

                if result || cost == 0
                {
                    hero_entity.add_card(CardItem
                    {
                        card_definition_id: item_definition_id as u16,
                        slot : 0,
                        card_unique_id: 0,
                    });// add item currency
                }

                tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
                heros_summary.push(hero_entity.clone_for_sending());
            },
            _ => 
            {
                cli_log::info!("error buying item");
            }
        }
    }
    else if inventory_type == 2
    {
        let cost  = map.definitions.equipment.get(item_definition_id as usize).map(|d| d.store_cost);
        cli_log::info!("equipment cost {cost:?}");
        match (hero_option, cost)
        {
            (Some(hero_entity), Some(cost)) =>
            {
                let result = hero_entity.remove_inventory_item(InventoryItem
                {
                    item_id : 0,
                    equipped : 0,
                    amount : cost * amount,
                });// remove soft currency

                if result || cost == 0
                {
                    hero_entity.add_equipment(EquipmentItem
                    {
                        equipment_definition_id: item_definition_id as u16,
                        slot : 0,
                        equipment_unique_id : 0
                    });// add item currency
                }

                tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
                heros_summary.push(hero_entity.clone_for_sending());
            },
            _ => 
            {
                cli_log::info!("error buying item");
            }
        }
    }

}

pub async fn sell_item(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    heros_summary : &mut Vec<HeroEntity>,
    item_id : u32,
    item_definition_id : u32,
    inventory_type : u8,
    player_id: u16,
    amount: u16)
{
    let mut hero_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
    let hero_option = hero_entities.get_mut(&player_id);

    if inventory_type == 0
    {
        let cost  = map.definitions.items.get(item_definition_id as usize).map(|d| d.cost);
        match (hero_option, cost) 
        {
            (Some(hero_entity), Some(cost)) => 
            {
                let result = hero_entity.remove_inventory_item(InventoryItem
                {
                    item_id : item_definition_id,
                    equipped:0,
                    amount,
                });

                // add soft currency
                if result 
                {
                    hero_entity.add_inventory_item(InventoryItem
                    {
                        item_id: 0,
                        equipped: 0,
                        amount: amount * cost,
                    });// add soft currency
                }

                tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
                heros_summary.push(hero_entity.clone_for_sending());
            },
            _ => 
            {
                cli_log::info!("error selling item")
            }
        }
    }
    else if inventory_type == 1
    {
        let cost  = map.definitions.cards.get(item_definition_id as usize).map(|d| d.store_cost);
        match (hero_option, cost) 
        {
            (Some(hero_entity), Some(cost)) => 
            {
                let result = hero_entity.remove_card(item_id);

                // add soft currency
                if result 
                {
                    hero_entity.add_inventory_item(InventoryItem
                    {
                        item_id: 0,
                        equipped: 0,
                        amount: amount * cost,
                    });// add soft currency
                }

                tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
                heros_summary.push(hero_entity.clone_for_sending());
            },
            _ => 
            {
                cli_log::info!("error selling card")
            }
        }
    }
    else if inventory_type == 2
    {
        let cost  = map.definitions.equipment.get(item_definition_id as usize).map(|d| d.store_cost);
        match (hero_option, cost)
        {
            (Some(hero_entity), Some(cost)) =>
            {
                let result = hero_entity.remove_equipment(item_id);

                // add soft currency
                if result
                {
                    hero_entity.add_inventory_item(InventoryItem
                    {
                        item_id: 0,
                        equipped: 0,
                        amount: amount * cost,
                    });// add soft currency
                }

                tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
                heros_summary.push(hero_entity.clone_for_sending());
            },
            _ =>
            {
                cli_log::info!("error selling equipment")
            }
        }
    }
}

pub async fn respawn(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    heros_summary : &mut Vec<HeroEntity>,
    player_id: u16,
    respawn_tile_id: TetrahedronId)
{
    let mut hero_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
    let hero_option = hero_entities.get_mut(&player_id);

    cli_log::info!("respawn {} to {}", player_id, respawn_tile_id.to_string());
    if let Some(hero_entity) = hero_option 
    {
        let character_definition = map.definitions.character_progression.get(hero_entity.level as usize).unwrap();
        cli_log::info!("b-respawn {}", character_definition.constitution);
        let updated_hero_entity = HeroEntity 
        {
            action: 0,
            time:0,
            health: character_definition.constitution,
            version: hero_entity.version + 1,
            position: respawn_tile_id.clone(),
            second_position : respawn_tile_id,
            path:[0,0,0,0,0,0],
            ..hero_entity.clone()
        };

        *hero_entity = updated_hero_entity;
        tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
        heros_summary.push(hero_entity.clone_for_sending());
    }
}

pub async fn move_character(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    heros_summary : &mut Vec<HeroEntity>,
    current_time : u64,
    player_id: u16,
    pos: TetrahedronId,
    second_pos: TetrahedronId,
    vertex_id: i32,
    path: [u8;6],
)
{
    let mut hero_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
    let hero_option = hero_entities.get_mut(&player_id);

    let current_time_in_seconds = (current_time / 1000) as u32;

    cli_log::info!("move {} vertex id {}", player_id, vertex_id);
    if let Some(hero_entity) = hero_option
    {
        let updated_hero_entity = HeroEntity
        {
            action: hero_command::WALK_ACTION,
            version: hero_entity.version + 1,
            position: pos,
            second_position: second_pos,
            vertex_id,
            path,
            time: current_time_in_seconds,
            ..hero_entity.clone()
        };

        // dash is deprecated, I don't care about it.
        // updated_hero_entity.set_flag(DASH_FLAG, dash);

        *hero_entity = updated_hero_entity;
        tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
        heros_summary.push(hero_entity.clone_for_sending());
    }
}

pub async fn set_action(
    map : &Arc<GameMap>,
    current_time : u64,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    heros_summary : &mut Vec<HeroEntity>,
    player_id: u16,
    action : u8
)
{
    let mut hero_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
    let hero_option = hero_entities.get_mut(&player_id);

    // cli_log::info!("set action {} {action}", player_id);
    if let Some(hero_entity) = hero_option 
    {
        let mut action = action;
        if action == hero_command::TOUCH 
        {
            action = hero_entity.action;
        }
        else if action == hero_command::TYPING
        {
            action = hero_entity.action;
            hero_entity.set_flag(CHAT_FLAG, true);
        }
        else if action == hero_command::NOT_TYPING
        {
            action = hero_entity.action;
            hero_entity.set_flag(CHAT_FLAG, false);
        }
        else
        {
            hero_entity.set_flag(CHAT_FLAG, false);
        }

        // cli_log::info!("flags {}", hero_entity.flags);
        hero_entity.action = action;
        hero_entity.version += 1;

        let current_time_in_seconds = (current_time / 1000) as u32;
        hero_entity.removed_expired_buffs(current_time_in_seconds);

        tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
        heros_summary.push(hero_entity.clone_for_sending());
    }
}

pub async fn greet(
    map : &Arc<GameMap>,
    heros_presentation_summary : &mut Vec<HeroPresentation>,
    player_id: u16
)
{
    let mut hero_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
    let hero_option = hero_entities.get_mut(&player_id);
    if let Some(hero_entity) = hero_option 
    {
        let name_with_padding = format!("{: <5}", hero_entity.hero_name);
        let name_data : Vec<u32> = name_with_padding.chars().into_iter().map(|c| c as u32).collect();
        let mut name_array = [0u32; 5];
        name_array.clone_from_slice(&name_data.as_slice()[0..5]);
        let hero_presentation = HeroPresentation 
        {
            player_id: hero_entity.hero_id,
            character_name: name_array,
        };

        heros_presentation_summary.push(hero_presentation);
    }
}

pub async fn activate_buff(
    map : &Arc<GameMap>,
    current_time : u64,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    heros_summary : &mut Vec<HeroEntity>,
    card_id : u32,
    player_id: u16)
{
    // cli_log::info!("---- activate buff with card {card_id}");
    let mut hero_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
    if let Some(hero) = hero_entities.get_mut(&player_id)
    {
        let current_time_in_seconds = (current_time / 1000) as u32;
        hero.removed_expired_buffs(current_time_in_seconds);
        let card = map.definitions.cards.get(card_id as usize).unwrap();

        let buff = map.definitions.buffs.get(&card.buff).unwrap();
        let result = hero.add_buff(buff.code, current_time_in_seconds, &map.definitions);
        // let result = hero_entity.equip_inventory_item(item_id, current_slot, new_slot);
        cli_log::info!("activate buff with id:{}",buff.id);

        if result 
        {
            hero.version += 1;
            tx_pe_gameplay_longterm.send(hero.clone()).await.unwrap();
            heros_summary.push(hero.clone_for_sending());
        }
    }

    
    cli_log::info!("--- activate buff");
    // match hero_option 
    // {
    //     Some(hero_entity) => 
    //     {
    //         let result = hero_entity.add_buff(card_id, &map.definitions);
    //         // let result = hero_entity.equip_inventory_item(item_id, current_slot, new_slot);
    //         // cli_log::info!("equip item with result {}",result);

    //         if result 
    //         {
    //             hero_entity.version += 1;
    //             tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
    //             heros_summary.push(hero_entity.clone());
    //         }
    //     },
    //     _ => 
    //     {
    //         cli_log::info!("error equipping item");
    //     }
    // }
}


pub async fn attack_character(
    map : &Arc<GameMap>,
    current_time: u64,
    _server_state: &Arc<ServerState>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    characters_summary : &mut Vec<HeroEntity>,
    attack_details_summary : &mut Vec<AttackResult>,
    characters_rewards_summary : &mut Vec<HeroReward>,
    card_id : u32,
    character_id: u16,
    other_character_id:u16,
    missed: u8)
{
    let mut character_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;

    let attacker_option= character_entities.get(&character_id);
    let defender_option= character_entities.get(&other_character_id);

    let current_time_in_seconds = (current_time / 1000) as u32;
    if let (Some(attacker), Some(defender)) = (attacker_option, defender_option)
    {

        let mut attacker = attacker.clone();
        let mut defender = defender.clone();

        let result = super::utils::attack::<HeroEntity, HeroEntity>(&map.definitions, card_id, current_time_in_seconds, missed, &mut attacker, &mut defender);

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

        if let Some(character) = character_entities.get_mut(&other_character_id)
        {
            *character = defender;
        }

        drop(character_entities);

        characters_summary.push(attacker_stored.clone_for_sending());
        characters_summary.push(defender_stored.clone_for_sending());

        attack_details_summary.push(AttackResult
        {
            id: (current_time % 10000) as u16,
            card_id,
            attacker_mob_id: 0,
            attacker_character_id: character_id,
            target_character_id: other_character_id,
            target_mob_id: 0,
            battle_type: BATTLE_CHAR_CHAR,
            result,
            target_tile_id: TetrahedronId::default(),
        });

        tx_pe_gameplay_longterm.send(attacker_stored).await.unwrap();
        tx_pe_gameplay_longterm.send(defender_stored).await.unwrap();
    }
}

pub async fn increase_skill_rank(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    heros_summary : &mut Vec<HeroEntity>,
    player_id: u16,
    skill_id: u8)
{
    let mut hero_entities : tokio::sync::MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
    let hero_option = hero_entities.get_mut(&player_id);


    if let Some(hero_entity) = hero_option
    {
        let profession = map.definitions.professions_by_id.get(hero_entity.profession as usize).unwrap();
        let skill_definition =  map.definitions.skills_by_id.get(skill_id as usize).unwrap();
        let costs_points = profession.get_skill_cost_and_points(&skill_definition.name).map_or((0,0,0), |s| (s.points, s.first_rank_cost, s.second_rank_cost));
        let count = hero_entity.get_skill_count(skill_id).unwrap_or(10);

        let cost = if count == 0 
        {
            costs_points.1
        }
        else if count == 1
        {
            costs_points.2
        }
        else 
        {
            255
        } as u8;

        if hero_entity.available_skill_points >= cost
        {
            hero_entity.available_skill_points -= cost;
            hero_entity.increase_skill_rank(skill_id, costs_points.0 as u8);
            tx_pe_gameplay_longterm.send(hero_entity.clone()).await.unwrap();
            heros_summary.push(hero_entity.clone_for_sending());
        }
        else
        {
            cli_log::info!("not enough skill points for player {}", player_id);
        }
    }
}

pub async fn disconnect(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &GaiaSender<HeroEntity>,
    characters_summary : &mut Vec<HeroEntity>,
    character_id: u16)
{
    let mut character_entities : tokio::sync:: MutexGuard<HashMap<u16, HeroEntity>> = map.character.lock().await;
    let character_option = character_entities.get_mut(&character_id);

    if let Some(character_entity) = character_option
    {
        character_entity.action = 0;
        character_entity.version += 1;
        tx_pe_gameplay_longterm.send(character_entity.clone()).await.unwrap();
        characters_summary.push(character_entity.clone_for_sending());
    }
}