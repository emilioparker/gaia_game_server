use std::{sync::Arc, collections::HashMap};
use tokio::{sync::{mpsc::Sender, Mutex}, time::error::Elapsed};
use crate::{character::{character_attack::CharacterAttack, character_command::{self, CharacterCommand, CharacterCommandInfo, CharacterMovement}, character_entity::{self, CharacterEntity, InventoryItem}, character_presentation::CharacterPresentation}, definitions::items::ItemUsage, gameplay_service::tile_commands_processor::attack_walker, map::{tetrahedron_id::TetrahedronId, GameMap}};

pub async fn process_player_commands (
    map : Arc<GameMap>,
    current_time : u64,
    player_commands_processor_lock : Arc<Mutex<Vec<CharacterCommand>>>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    players_presentation_summary : &mut Vec<CharacterPresentation>,
    player_attacks_summary : &mut  Vec<CharacterAttack>,
    delayed_player_commands_lock : Arc<Mutex<Vec<(u64, CharacterCommand)>>>
)
{
    let mut player_commands_data = player_commands_processor_lock.lock().await;

    if player_commands_data.len() == 0
    {
        return;
    }

    for player_command in player_commands_data.iter()
    {
        let cloned_data = player_command.to_owned();
        if let Some(atomic_time) = map.active_players.get(&cloned_data.player_id)
        {
            atomic_time.store(current_time, std::sync::atomic::Ordering::Relaxed);
        }

        match &player_command.info 
        {
            character_command::CharacterCommandInfo::Touch() => todo!(),
            character_command::CharacterCommandInfo::Movement(movement_data) => 
            {
                move_character(&map, tx_pe_gameplay_longterm, players_summary, cloned_data.player_id, movement_data.position.clone(), movement_data.second_position.clone()).await;
            },
            character_command::CharacterCommandInfo::SellItem(_faction, item_id, amount) => 
            {
                sell_item(&map, tx_pe_gameplay_longterm, players_summary, *item_id, cloned_data.player_id, *amount).await
            },
            character_command::CharacterCommandInfo::BuyItem(_faction, item_id, amount) => 
            {
                buy_item(&map, tx_pe_gameplay_longterm, players_summary, *item_id, cloned_data.player_id, *amount).await
            },
            character_command::CharacterCommandInfo::UseItem(_faction, item_id, amount) => 
            {
                use_item(&map, tx_pe_gameplay_longterm, players_summary, *item_id, cloned_data.player_id, *amount).await;
            },
            character_command::CharacterCommandInfo::EquipItem(equip_data) => 
            {
                equip_item(&map, tx_pe_gameplay_longterm, players_summary, equip_data.item_id, cloned_data.player_id, equip_data.current_slot,equip_data.new_slot).await;
            },
            character_command::CharacterCommandInfo::Respawn() => 
            {
                respawn(&map, tx_pe_gameplay_longterm, players_summary, cloned_data.player_id).await;
            },
            character_command::CharacterCommandInfo::Action(action) => 
            {
                set_action(&map, tx_pe_gameplay_longterm, players_summary, cloned_data.player_id, *action).await;
            },
            character_command::CharacterCommandInfo::Greet() => 
            {
                greet(&map, players_presentation_summary, cloned_data.player_id).await;
            },
            character_command::CharacterCommandInfo::ActivateBuff(card_id) => 
            {
                activate_buff(&map, tx_pe_gameplay_longterm, players_summary, *card_id, cloned_data.player_id).await;
            },
            character_command::CharacterCommandInfo::AttackCharacter(other_player_id, card_id, required_time, active_effect) => 
            {
                let end_time = current_time + *required_time as u64;
                if *required_time == 0
                {
                    attack_character(&map, tx_pe_gameplay_longterm, players_summary, *card_id, cloned_data.player_id, *other_player_id).await;
                }
                else 
                {
                    println!("------------ required time for player attack {required_time} current time: {current_time} {card_id}");
                    let mut lock = delayed_player_commands_lock.lock().await;
                    let info = CharacterCommandInfo::AttackCharacter(*other_player_id, *card_id, *required_time, *active_effect);
                    let character_action = CharacterCommand { player_id: cloned_data.player_id, info };
                    lock.push((end_time, character_action));
                    drop(lock);
                }

                let attack = CharacterAttack
                {
                    id: (current_time % 10000) as u16,
                    player_id: cloned_data.player_id,
                    target_player_id: *other_player_id,
                    card_id: *card_id,
                    target_tile_id: TetrahedronId::from_string("A"),
                    required_time: *required_time,
                    active_effect: *active_effect
                };

                println!("--- attack player {} at {} effect {}", other_player_id, attack.required_time, attack.active_effect);
                player_attacks_summary.push(attack);
            },
        }
    }
    player_commands_data.clear();
}


pub async fn process_delayed_player_commands(
    map : Arc<GameMap>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    characters_summary : &mut Vec<CharacterEntity>,
    delayed_character_commands_to_execute : Vec<CharacterCommand>,
)
{
    if delayed_character_commands_to_execute.len() == 0
    {
        return;
    }

    for player_command in delayed_character_commands_to_execute.iter()
    {
        match &player_command.info 
        {
            character_command::CharacterCommandInfo::AttackCharacter(other_character_id, card_id, _required_time, _active_effect) => 
            {
                attack_character(&map, tx_pe_gameplay_longterm, characters_summary, *card_id, player_command.player_id, *other_character_id).await;
            },
            _ => 
            {
                println!("delayed command not supported");
            }
        }
    }
}

pub async fn use_item(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    item_id : u32,
    player_id: u16,
    amount: u16)
{
    let item_definition = map.definitions.items.get(item_id as usize);

    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.character.lock().await;
    let player_option = player_entities.get_mut(&player_id);

    match (player_option, item_definition) 
    {
        (Some(player_entity), Some(definition)) => 
        {
            let character_definition = map.definitions.character_progression.get(player_entity.level as usize).unwrap();
            if definition.usage != 0
            {
                let result = player_entity.remove_inventory_item(InventoryItem
                {
                    item_id,
                    equipped: 0,
                    amount,
                });// remove soft currency

                println!("using item with result {} and  {:?}",result, definition.usage);

                match (result, definition.usage)
                {
                    (true, usage) if usage == ItemUsage::Heal as u8 =>  // heal
                    {
                        player_entity.health = u16::min(character_definition.constitution, player_entity.health + 5);
                        player_entity.version += 1;
                    },
                    (true, usage) if usage == ItemUsage::AddXp as u8 =>  // heal
                    {
                        player_entity.available_skill_points += 2;
                        player_entity.version += 1;
                    },
                    _ => 
                    {
                        println!("item {} cannot be used ", item_id);
                    }
                }
            }

            // println!("Add health {:?}", player_entity);
            tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
            players_summary.push(player_entity.clone());
        },
        _ => 
        {
            println!("error buying item");
        }
    }
}

pub async fn equip_item(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    item_id : u32,
    player_id: u16,
    current_slot: u8,
    new_slot:u8)
{
    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.character.lock().await;
    let player_option = player_entities.get_mut(&player_id);

    match player_option 
    {
        Some(player_entity) => 
        {
            let result = player_entity.equip_inventory_item(item_id, current_slot, new_slot);
            println!("equip item with result {}",result);

            tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
            players_summary.push(player_entity.clone());
        },
        _ => 
        {
            println!("error equipping item");
        }
    }
}

pub async fn buy_item(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    item_id : u32,
    player_id: u16,
    amount: u16)
{
    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.character.lock().await;
    println!("Buy item with id {item_id}");
    let mut is_card  = false;
    let cost  = if item_id < 10000
    {
        map.definitions.items.get(item_id as usize).map(|d| d.cost)
    }
    else if item_id >= 10000
    {
        is_card = true;
        map.definitions.get_card(item_id as usize).map(|d| d.cost)
    }
    else
    {
        None
    };

    println!("cost {cost:?}");
    let player_option = player_entities.get_mut(&player_id);

    match (player_option, cost) 
    {
        (Some(player_entity), Some(cost)) => 
        {
            if is_card && player_entity.has_inventory_item(item_id)
            {
                println!("can't purchase more than one card of the same type")
            }
            else
            {
                    
                let result = player_entity.remove_inventory_item(InventoryItem
                {
                    item_id : 0,
                    equipped : 0,
                    amount : cost * amount,
                });// remove soft currency

                if result || cost == 0
                {
                    player_entity.add_inventory_item(InventoryItem
                    {
                        item_id,
                        equipped : 0,
                        amount
                    });// add item currency
                }

                tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
                players_summary.push(player_entity.clone());
            }
        },
        _ => 
        {
            println!("error buying item");
        }
    }
}

pub async fn sell_item(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    item_id : u32,
    player_id: u16,
    amount: u16)
{
    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.character.lock().await;
    let item_definition = map.definitions.items.get(item_id as usize);
    let player_option = player_entities.get_mut(&player_id);

    match (player_option, item_definition) 
    {
        (Some(player_entity), Some(definition)) => {
            let result = player_entity.remove_inventory_item(InventoryItem
            {
                item_id : item_id,
                equipped:0,
                amount,
            });// add soft currency

            if result 
            {
                player_entity.add_inventory_item(InventoryItem
                {
                    item_id: 0,
                    equipped: 0,
                    amount: amount * definition.cost,
                });// add soft currency
            }

            tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
            players_summary.push(player_entity.clone());

        },
        _ => 
        {
            println!("error selling item");
        }
    }
}

pub async fn respawn(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    player_id: u16)
{
    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.character.lock().await;
    let player_option = player_entities.get_mut(&player_id);

    println!("respawn {}", player_id);
    if let Some(player_entity) = player_option 
    {
        let character_definition = map.definitions.character_progression.get(player_entity.level as usize).unwrap();
        println!("b-respawn {}", character_definition.constitution);
        let updated_player_entity = CharacterEntity 
        {
            action: 0,
            health: character_definition.constitution,
            version: player_entity.version + 1,
            ..player_entity.clone()
        };

        *player_entity = updated_player_entity;
        tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
        players_summary.push(player_entity.clone());
    }
}

pub async fn move_character(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    player_id: u16,
    pos:TetrahedronId,
    next_pos:TetrahedronId,
)
{
    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.character.lock().await;
    let player_option = player_entities.get_mut(&player_id);

    println!("move {}", player_id);
    if let Some(player_entity) = player_option 
    {
        let updated_player_entity = CharacterEntity 
        {
            action: character_command::WALK_ACTION,
            version: player_entity.version + 1,
            position: pos,
            second_position: next_pos,
            ..player_entity.clone()
        };

        *player_entity = updated_player_entity;
        tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
        players_summary.push(player_entity.clone());
    }
}

pub async fn set_action(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    player_id: u16,
    action : u32
)
{
    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.character.lock().await;
    let player_option = player_entities.get_mut(&player_id);

    println!("set action {} {action}", player_id);
    if let Some(player_entity) = player_option 
    {
        let mut action = action;
        if action == character_command::TOUCH 
        {
            action = player_entity.action;
        }

        let updated_player_entity = CharacterEntity 
        {
            action,
            version: player_entity.version + 1,
            ..player_entity.clone()
        };

        *player_entity = updated_player_entity;
        tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
        players_summary.push(player_entity.clone());
    }
}

pub async fn greet(
    map : &Arc<GameMap>,
    players_presentation_summary : &mut Vec<CharacterPresentation>,
    player_id: u16
)
{
    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.character.lock().await;
    let player_option = player_entities.get_mut(&player_id);
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

pub async fn activate_buff(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    players_summary : &mut Vec<CharacterEntity>,
    card_id : u32,
    player_id: u16)
{
    let mut player_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.character.lock().await;
    let player_option = player_entities.get_mut(&player_id);

    println!("--- activate buff");
    match player_option 
    {
        Some(player_entity) => 
        {
            let result = player_entity.add_buff(card_id, &map.definitions);
            // let result = player_entity.equip_inventory_item(item_id, current_slot, new_slot);
            // println!("equip item with result {}",result);

            if result 
            {
                tx_pe_gameplay_longterm.send(player_entity.clone()).await.unwrap();
                players_summary.push(player_entity.clone());
            }
        },
        _ => 
        {
            println!("error equipping item");
        }
    }
}

pub async fn attack_character(
    map : &Arc<GameMap>,
    tx_pe_gameplay_longterm : &Sender<CharacterEntity>,
    characters_summary : &mut Vec<CharacterEntity>,
    card_id : u32,
    character_id: u16,
    other_character_id:u16)
{
    let mut character_entities : tokio::sync:: MutexGuard<HashMap<u16, CharacterEntity>> = map.character.lock().await;
    let character_option = character_entities.get_mut(&character_id);

    let mut character_attack = 0u16;
    if let Some(character_entity) = character_option 
    {
        if let Some(card_definition) = map.definitions.get_card(card_id as usize)
        {
            let attack = character_entity.get_strength(card_definition.strength_factor) as f32;
            character_attack = attack.round() as u16;
            character_entity.use_buffs(vec![crate::character::character_entity::Stat::Strength]);
        }
        else
        {
            println!("-- card id not found {card_id}");
            return;
        }
        tx_pe_gameplay_longterm.send(character_entity.clone()).await.unwrap();
        characters_summary.push(character_entity.clone());
    }

    let other_character_option = character_entities.get_mut(&other_character_id);

    if let Some(character_entity) = other_character_option 
    {
        let defense = character_entity.get_defense(0f32) as f32;
        let character_defense = defense.round() as u16;
        character_entity.use_buffs(vec![crate::character::character_entity::Stat::Defense]);

        let damage = character_attack.saturating_sub(character_defense);

        character_entity.health = i16::max(0, damage as i16 - character_attack as i16) as u16;
        tx_pe_gameplay_longterm.send(character_entity.clone()).await.unwrap();
        characters_summary.push(character_entity.clone());
    }
}