
use std::collections::{HashSet, HashMap};
use std::sync::Arc;
use crate::long_term_storage_service::db_character::{StoredCharacter, StoredInventoryItem};
use crate::map::GameMap;
use crate::character::character_entity::{CharacterEntity, InventoryItem};
use bson::doc;
use bson::oid::ObjectId;
use mongodb::Client;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver};
use futures_util::stream::StreamExt;


pub async fn get_characters_from_db_by_world(
    world_id : Option<ObjectId>,
    db_client : Client
) -> HashMap<u16, CharacterEntity> {
    println!("get players from db using {:?}", world_id);

    let mut data = HashMap::<u16, CharacterEntity>::new();

    let data_collection: mongodb::Collection<StoredCharacter> = db_client.database("game").collection::<StoredCharacter>("characters");

    let mut cursor = data_collection
    .find(
        doc! {
                "world_id": world_id
        },
        None,
    ).await
    .unwrap();

    let mut count = 0;
    while let Some(result) = cursor.next().await {
        match result 
        {
            Ok(doc) => {

                let inventory = doc.inventory.into_iter().map(|item| InventoryItem {
                    item_id: item.item_id,
                    level: item.level,
                    quality: item.quality,
                    amount: item.amount,
                }).collect();

                let player =  CharacterEntity
                {
                    character_id: doc.character_id,
                    player_id: doc.player_id,
                    version:doc.version,
                    faction: crate::get_faction_code(&doc.faction),
                    object_id: doc.id,
                    position: doc.position,
                    second_position: doc.position,
                    action: 0,
                    character_name: doc.character_name,
                    inventory,
                    inventory_hash: 1,
                    level: doc.level,
                    experience: doc.experience,
                    available_skill_points: doc.available_skill_points,
                    constitution: doc.constitution,
                    strenght: doc.strenght,
                    dexterity: doc.dexterity,
                    intelligence: doc.intelligence,
                    health: doc.health,
                };
                count += 1;
                data.insert(doc.character_id, player);
            },
            Err(error_details) => {
                println!("error getting characters from db with {:?}", error_details);
            },
        }
    }
    println!("Got {} characters from database", count);

    data
}

pub fn start_server(
    mut rx_pe_realtime_longterm : Receiver<CharacterEntity>,
    map : Arc<GameMap>,
    db_client : Client){

    let modified_players = HashSet::<u16>::new();
    let modified_players_reference = Arc::new(Mutex::new(modified_players));

    let modified_players_update_lock = modified_players_reference.clone();
    let modified_players_reader_lock = modified_players_reference.clone();

    let map_reader = map.clone();
    let map_updater = map.clone();


    // we keep track of which players have change in a hashset
    // we also save the changed players
    tokio::spawn(async move {
        loop {
            let message = rx_pe_realtime_longterm.recv().await.unwrap();
            // println!("player entity changed  with inventory ? {}" , message.inventory.len());
            let mut modified_players = modified_players_update_lock.lock().await;
            modified_players.insert(message.character_id.clone());

            let mut locked_players = map_updater.players.lock().await;

            let old = locked_players.get(&message.character_id);
            match old {
                Some(_previous_record) => {
                    locked_players.insert(message.character_id.clone(), message);
                }
                _ => {
                   locked_players.insert(message.character_id.clone(), message);
                }
            }
            // we need to save into the hashmap and then save to a file.
        }
    });

    // after a few seconds we try to save all changes to the database.
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
            let mut modified_player_keys = modified_players_reader_lock.lock().await;
            let locked_players = map_reader.players.lock().await;

            let mut modified_player_entities = Vec::<CharacterEntity>::new();
            for player_id in modified_player_keys.iter(){
                println!("this player was changed {}", player_id.to_string());
                if let Some(player_data) = locked_players.get(player_id) {
                    modified_player_entities.push(player_data.clone());
                }
            }

            modified_player_keys.clear();
            drop(modified_player_keys);
            drop(locked_players);

            let data_collection: mongodb::Collection<StoredCharacter> = db_client.database("game").collection::<StoredCharacter>("characters");

            for player in modified_player_entities {
                let updated_inventory : Vec<StoredInventoryItem> = player.inventory
                .into_iter()
                .map(|item| StoredInventoryItem ::from(item))
                .collect();

                let serialized_data= bson::to_bson(&updated_inventory).unwrap();
                let serialized_position= bson::to_bson(&player.second_position).unwrap();

                let update_result = data_collection.update_one(
                    doc! {
                        "_id": player.object_id,
                    },
                    doc! {
                        "$set": {
                            "position":serialized_position,
                            "inventory" : serialized_data,
                            "level": bson::to_bson(&player.level).unwrap(),
                            "experience" : bson::to_bson(&player.experience).unwrap(),
                            "available_skill_points": bson::to_bson(&player.available_skill_points).unwrap(),
                            "health": bson::to_bson(&player.health).unwrap(),
                            "constitution": bson::to_bson(&player.constitution).unwrap(),
                            "strenght": bson::to_bson(&player.strenght).unwrap(),
                            "dexterity": bson::to_bson(&player.dexterity).unwrap(),
                            "intelligence": bson::to_bson(&player.intelligence).unwrap(),
                        }
                    },
                    None
                ).await;

                println!("updated player result {:?}", update_result);
            }
        }
    });
}



