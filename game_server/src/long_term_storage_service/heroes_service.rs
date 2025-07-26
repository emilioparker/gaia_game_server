
use std::collections::{HashSet, HashMap};
use std::sync::Arc;
use crate::buffs::buff::{Buff, BuffUser};
use crate::hero::hero_card_inventory::CardItem;
use crate::hero::hero_inventory::InventoryItem;
use crate::hero::hero_tower_progress::HeroTowerProgress;
use crate::hero::hero_weapon_inventory::WeaponItem;
use crate::long_term_storage_service::db_hero::{StoredBuff, StoredHero, StoredInventoryItem, StoredTowerProgress};
use crate::map::tetrahedron_id::TetrahedronId;
use crate::map::GameMap;
use crate::hero::hero_entity::HeroEntity;
use crate::ServerState;
use bson::doc;
use bson::oid::ObjectId;
use mongodb::Client;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver};
use futures_util::stream::StreamExt;


pub async fn get_heroes_from_db_by_world(
    world_id : Option<ObjectId>,
    db_client : Client
) -> HashMap<u16, HeroEntity> {
    cli_log::info!("get heroes from db using {:?}", world_id);

    let mut data = HashMap::<u16, HeroEntity>::new();

    let data_collection: mongodb::Collection<StoredHero> = db_client.database("game").collection::<StoredHero>("characters");

    let mut cursor = data_collection
    .find(
        doc! {
                "world_id": world_id
        },
        None,
    ).await
    .unwrap();

    let mut count = 0;
    while let Some(result) = cursor.next().await 
    {
        match result 
        {
            Ok(doc) => 
            {
                let inventory = doc.inventory.into_iter().map(|item| InventoryItem 
                {
                    item_id: item.item_id,
                    equipped: item.equipped,
                    amount: item.amount,
                }).collect();

                let card_inventory = doc.card_inventory.into_iter().map(|item| CardItem 
                {
                    card_id: item.item_id,
                    equipped: item.equipped,
                    amount: item.amount,
                }).collect();

                let weapon_inventory = doc.weapon_inventory.into_iter().map(|item| WeaponItem 
                {
                    weapon_id: item.item_id,
                    equipped: item.equipped,
                    amount: item.amount,
                }).collect();

                let buffs : Vec<Buff> = doc.buffs.into_iter().map(|stored_buff| stored_buff.into()).collect();
                let buffs_summary : [u8;5]= [0,0,0,0,0];

                let tower_progress :  HeroTowerProgress = doc.tower_progress.into();


                cli_log::info!("----- faction {}", doc.faction);
                let pos = TetrahedronId::from_string(&doc.position);
                let mut player =  HeroEntity
                {
                    hero_id: doc.character_id,
                    player_id: doc.player_id,
                    version:doc.version,
                    faction: doc.faction,
                    object_id: doc.id,
                    position: pos.clone(),
                    second_position: pos.clone(),
                    vertex_id: doc.vertex_id,
                    path: [0,0,0,0,0,0],
                    time:0,
                    action: doc.action,
                    flags: doc.flags,
                    hero_name: doc.character_name,
                    inventory,
                    card_inventory,
                    weapon_inventory,
                    inventory_version: 1,
                    level: doc.level,
                    experience: doc.experience,
                    available_skill_points: doc.available_skill_points,
                    weapon:doc.weapon,
                    strength_points: doc.strength_points,
                    defense_points: doc.defense_points,
                    intelligence_points: doc.intelligence_points,
                    mana_points: doc.mana_points,
                    base_defense: doc.defense,
                    base_strength: doc.strength,
                    base_intelligence: doc.intelligence,
                    base_mana: doc.mana,
                    health: doc.health,
                    buffs,
                    buffs_summary,
                    tower_progress,
                };
                player.summarize_buffs();

                count += 1;
                data.insert(doc.character_id, player);
            },
            Err(error_details) => {
                cli_log::info!("error getting characters from db with {:?}", error_details);
            },
        }
    }
    cli_log::info!("Got {} characters from database", count);

    data
}

pub fn start_server(
    mut rx_he_realtime_longterm : Receiver<HeroEntity>,
    map : Arc<GameMap>,
    server_state : Arc<ServerState>,
    db_client : Client)
    {

    let modified_players = HashSet::<u16>::new();
    let modified_players_reference = Arc::new(Mutex::new(modified_players));

    let modified_players_update_lock = modified_players_reference.clone();
    let modified_players_reader_lock = modified_players_reference.clone();

    let map_reader = map.clone();
    let map_updater = map.clone();

    let map_reader_server_state = server_state.clone();
    let map_updater_server_state = server_state.clone();


    // we keep track of which players have change in a hashset
    // we also save the changed players
    tokio::spawn(async move 
    {
        loop 
        {
            let message = rx_he_realtime_longterm.recv().await.unwrap();
            cli_log::info!("--hero entity changed  with id {}" , message.hero_id);
            let mut modified_players = modified_players_update_lock.lock().await;
            modified_players.insert(message.hero_id.clone());

            let mut locked_players = map_updater.character.lock().await;

            let old = locked_players.get(&message.hero_id);
            match old 
            {
                Some(_previous_record) => 
                {
                    locked_players.insert(message.hero_id.clone(), message);
                }
                _ => 
                {
                   locked_players.insert(message.hero_id.clone(), message);
                }
            }
            map_updater_server_state.pending_character_entities_to_save.store(modified_players.len() as u32, std::sync::atomic::Ordering::Relaxed);
        }
    });

    // after a few seconds we try to save all changes to the database.
    tokio::spawn(async move 
    {
        let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
        let current_time_in_millis = current_time.as_millis() as u64;
        map_reader_server_state.last_character_entities_save_timestamp.store(current_time_in_millis, std::sync::atomic::Ordering::Relaxed);

        loop 
        {
            tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
            let mut modified_player_keys = modified_players_reader_lock.lock().await;
            let modified_heroes = modified_player_keys.len();
            let locked_players = map_reader.character.lock().await;

            let mut modified_player_entities = Vec::<HeroEntity>::new();
            for player_id in modified_player_keys.iter()
            {
                cli_log::info!("this player was changed {}", player_id.to_string());
                if let Some(player_data) = locked_players.get(player_id) 
                {
                    modified_player_entities.push(player_data.clone());
                }
            }

            modified_player_keys.clear();
            drop(modified_player_keys);
            drop(locked_players);

            let data_collection: mongodb::Collection<StoredHero> = db_client.database("game").collection::<StoredHero>("characters");

            for player in modified_player_entities 
            {
                let inventory : Vec<StoredInventoryItem> = player.inventory
                .into_iter()
                .map(|item| StoredInventoryItem ::from(item))
                .collect();
                let inventory_serialized_data= bson::to_bson(&inventory).unwrap();

                let card_inventory : Vec<StoredInventoryItem> = player.card_inventory
                .into_iter()
                .map(|item| StoredInventoryItem ::from(item))
                .collect();
                let card_inventory_serialized_data= bson::to_bson(&card_inventory).unwrap();

                let weapon_inventory : Vec<StoredInventoryItem> = player.weapon_inventory
                .into_iter()
                .map(|item| StoredInventoryItem ::from(item))
                .collect();
                let weapon_inventory_serialized_data= bson::to_bson(&weapon_inventory).unwrap();

                let updated_buffs : Vec<StoredBuff> = player.buffs
                .into_iter()
                .map(|buff| StoredBuff ::from(buff))
                .collect();

                let tower_progress = StoredTowerProgress::from(player.tower_progress);

                let serialized_buffs_data= bson::to_bson(&updated_buffs).unwrap();
                let serialized_position= bson::to_bson(&player.second_position.to_string()).unwrap();

                let update_result = data_collection.update_one(
                    doc! 
                    {
                        "_id": player.object_id,
                    },
                    doc! 
                    {
                        "$set": 
                        {
                            "position":serialized_position,
                            "vertex_id": bson::to_bson(&player.vertex_id).unwrap(),
                            "action":bson::to_bson(&player.action).unwrap(),
                            "flags":bson::to_bson(&player.flags).unwrap(),
                            "inventory" : inventory_serialized_data,
                            "card_inventory" : card_inventory_serialized_data,
                            "weapon_inventory" : weapon_inventory_serialized_data,
                            "tower_progress" : bson::to_bson(&tower_progress).unwrap(),
                            "level": bson::to_bson(&player.level).unwrap(),
                            "experience" : bson::to_bson(&player.experience).unwrap(),
                            "available_skill_points": bson::to_bson(&player.available_skill_points).unwrap(),
                            "weapon": bson::to_bson(&player.weapon).unwrap(),
                            "defense_points": bson::to_bson(&player.defense_points).unwrap(),
                            "strength_points": bson::to_bson(&player.strength_points).unwrap(),
                            "mana_points": bson::to_bson(&player.mana_points).unwrap(),
                            "intelligence_points": bson::to_bson(&player.intelligence_points).unwrap(),
                            "health": bson::to_bson(&player.health).unwrap(),
                            "defense": bson::to_bson(&player.base_defense).unwrap(),
                            "strength": bson::to_bson(&player.base_strength).unwrap(),
                            "mana": bson::to_bson(&player.base_mana).unwrap(),
                            "intelligence": bson::to_bson(&player.base_intelligence).unwrap(),
                            "buffs" : serialized_buffs_data,
                        }
                    },
                    None
                ).await;

                map_reader_server_state.pending_character_entities_to_save.store(0, std::sync::atomic::Ordering::Relaxed);

                map_reader_server_state.saved_character_entities.fetch_add(modified_heroes as u32, std::sync::atomic::Ordering::Relaxed);

                let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
                let current_time_in_millis = current_time.as_millis() as u64;
                map_reader_server_state.last_character_entities_save_timestamp.store(current_time_in_millis, std::sync::atomic::Ordering::Relaxed);

                cli_log::info!("updated player result {:?}", update_result);
            }
        }
    });
}



