
use std::collections::{HashSet, HashMap};
use std::sync::Arc;
use crate::player::player_entity::PlayerEntity;
use bson::doc;
use mongodb::Client;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver};
use futures_util::stream::StreamExt;

use super::db_player::StoredPlayer;

// data is compressed
// pub async fn preload_db(
//     world_name : &str,
//     world_id: Option<ObjectId>,
//     players_data : HashMap<u64, StoredPlayer>,
//     db_client : Client
// ) {

//     let data_collection: mongodb::Collection<StoredPlayer> = db_client.database("game").collection::<StoredPlayer>("players");
//     let mut stored_players = Vec::<StoredPlayer>:: new();

//     for region in players_data
//     {
//         let bson = bson::Bson::Binary(bson::Binary {
//             subtype: bson::spec::BinarySubtype::Generic,
//             bytes: region.1,
//         });

//         let data = StoredRegion {
//             id : None,
//             world_id : world_id,
//             world_name : world_name.to_owned(),
//             region_id : region.0.to_string(),
//             compressed_data : bson
//         };

//         stored_regions.push(data);
//     }

//     let insert_result = data_collection.insert_many(stored_regions, None).await.unwrap();
//     println!("{:?}", insert_result);

// }

pub async fn get_players_from_db(
    world_name : &str,
    db_client : Client
) -> HashMap<u64, PlayerEntity> {

    let mut data = HashMap::<u64, PlayerEntity>::new();

    let data_collection: mongodb::Collection<StoredPlayer> = db_client.database("game").collection::<StoredPlayer>("players");

    // Look up one document:
    let mut cursor = data_collection
    .find(
        doc! {
                "world_name": world_name.to_owned()
        },
        None,
    ).await
    .unwrap();

    let mut count = 0;
    while let Some(Ok(doc)) = cursor.next().await {
        let player =  PlayerEntity{
            player_id: doc.player_id,
            object_id: doc.id,
            constitution: doc.constitution
        };
        count += 1;
        data.insert(doc.player_id, player);
    }
    println!("Got {} players from database", count);

    data
}

pub fn start_server(
    mut rx_pe_realtime_longterm : Receiver<PlayerEntity>,
    players : HashMap<u64, PlayerEntity>,
    db_client : Client
) {

    let modified_players = HashSet::<u64>::new();
    let modified_players_reference = Arc::new(Mutex::new(modified_players));

    let modified_players_update_lock = modified_players_reference.clone();
    let modified_players_reader_lock = modified_players_reference.clone();

    let players_reference = Arc::new(Mutex::new(players));
    let players_reader = players_reference.clone();
    let players_updater = players_reference.clone();


    // we keep track of which players have change in a hashset
    // we also save the changed players
    tokio::spawn(async move {
        loop {
            let message = rx_pe_realtime_longterm.recv().await.unwrap();
            println!("got a player changed {:?} ", message);

            let mut modified_players = modified_players_update_lock.lock().await;
            modified_players.insert(message.player_id.clone());

            let mut locked_players = players_updater.lock().await;

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
            let locked_players = players_reader.lock().await;

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

            let data_collection: mongodb::Collection<StoredPlayer> = db_client.database("game").collection::<StoredPlayer>("players");

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

                println!("updated region result {:?}", update_result);
            }
        }
    });
}



