
use std::collections::{HashSet, HashMap};
use std::sync::Arc;
use crate::long_term_storage_service::db_character::StoredCharacter;
use crate::map::GameMap;
use crate::player::player_entity::PlayerEntity;
use bson::doc;
use bson::oid::ObjectId;
use mongodb::Client;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver};
use futures_util::stream::StreamExt;


pub async fn get_players_from_db(
    world_id : Option<ObjectId>,
    db_client : Client
) -> HashMap<u64, PlayerEntity> {
    println!("get players from db using {:?}", world_id);

    let mut data = HashMap::<u64, PlayerEntity>::new();

    let data_collection: mongodb::Collection<StoredCharacter> = db_client.database("game").collection::<StoredCharacter>("players");

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
                let player =  PlayerEntity{
                    player_id: doc.player_id,
                    object_id: doc.id,
                    position: [0f32, 0f32, 0f32],
                    second_position: [0f32, 0f32, 0f32],
                    action: 0,
                    constitution: doc.constitution,
                    health: doc.health,
                    character_name: doc.character_name,
                };
                count += 1;
                data.insert(doc.player_id, player);
            },
            Err(error_details) => {
                println!("error getting player from db with {:?}", error_details);
            },
        }
    }
    println!("Got {} players from database", count);

    data
}

pub fn start_server(
    mut rx_pe_realtime_longterm : Receiver<PlayerEntity>,
    map : Arc<GameMap>,
    db_client : Client
) {

    let modified_players = HashSet::<u64>::new();
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

            let mut modified_players = modified_players_update_lock.lock().await;
            modified_players.insert(message.player_id.clone());

            let mut locked_players = map_updater.players.lock().await;

            let old = locked_players.get(&message.player_id);
            match old {
                Some(_previous_record) => {
                    locked_players.insert(message.player_id.clone(), message);
                }
                _ => {
                   locked_players.insert(message.player_id.clone(), message);
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

            let mut modified_player_entities = Vec::<PlayerEntity>::new();
            for player_id in modified_player_keys.iter(){
                println!("this player was changed {}", player_id.to_string());
                if let Some(player_data) = locked_players.get(player_id) {
                    modified_player_entities.push(player_data.clone());
                }
            }

            modified_player_keys.clear();
            drop(modified_player_keys);
            drop(locked_players);

            let data_collection: mongodb::Collection<StoredCharacter> = db_client.database("game").collection::<StoredCharacter>("players");

            for player in modified_player_entities {
                let update_result = data_collection.update_one(
                    doc! {
                        "_id": player.object_id,
                    },
                    doc! {
                        "$set": {"constitution": player.constitution + 1}
                    },
                    None
                ).await;

                println!("updated player result {:?}", update_result);
            }
        }
    });
}



