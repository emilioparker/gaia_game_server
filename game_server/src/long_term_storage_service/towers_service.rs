
use std::collections::{HashSet, HashMap};
use std::sync::Arc;
use crate::{get_faction_from_code, get_faction_code};
use crate::long_term_storage_service::db_tower::StoredDamageByFaction;
use crate::map::GameMap;
use crate::map::tetrahedron_id::TetrahedronId;
use crate::tower::tower_entity::{TowerEntity, DamageByFaction};
use bson::doc;
use bson::oid::ObjectId;
use mongodb::Client;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use futures_util::stream::StreamExt;

use super::db_tower::StoredTower;


pub async fn get_towers_from_db_by_world(
    world_id : Option<ObjectId>,
    db_client : Client
) -> HashMap<TetrahedronId, TowerEntity> {
    println!("get towers from db using {:?}", world_id);

    let mut data = HashMap::<TetrahedronId, TowerEntity>::new();

    let data_collection: mongodb::Collection<StoredTower> = db_client.database("game").collection::<StoredTower>("towers");

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
                let record = doc.damage_received_in_event.into_iter().map(|item| DamageByFaction 
                {
                    amount: item.amount,
                    event_id: item.event_id,
                    faction: get_faction_code(&item.faction),
                }).collect();

                let tower =  TowerEntity{
                    object_id: doc.id,
                    version:doc.version,
                    tetrahedron_id: TetrahedronId::from_string(&doc.tetrahedron_id),
                    event_id: doc.event_id,
                    faction:get_faction_code(&doc.faction),
                    total_damage: 0,
                    damage_received_in_event: record,
                };
                count += 1;
                data.insert(tower.tetrahedron_id.clone(), tower);
            },
            Err(error_details) => 
            {
                println!("error getting towers from db with {:?}", error_details);
            },
        }
    }
    println!("Got {} towerw from database", count);

    data
}

pub async fn preload_db(
    world_name : &str,
    world_id: Option<ObjectId>,
    towers : Vec<TetrahedronId>,
    db_client : Client
) {

    let data_collection: mongodb::Collection<StoredTower> = db_client.database("game").collection::<StoredTower>("towers");
    let mut stored_towers = Vec::<StoredTower>:: new();

    for tower_id in towers
    {
        let data = StoredTower {
            id : None,
            tetrahedron_id : tower_id.to_string(),
            world_id : world_id,
            world_name : world_name.to_owned(),
            version : 0,
            event_id : 0,
            faction : get_faction_from_code(0),
            damage_received_in_event : Vec::<StoredDamageByFaction>::new()
        };

        stored_towers.push(data);
    }

    let insert_result = data_collection.insert_many(stored_towers, None).await.unwrap();
    println!("{:?}", insert_result);

}
pub fn start_server(
    mut rx_te_realtime_longterm : Receiver<TowerEntity>,
    map : Arc<GameMap>,
    db_client : Client){

    let modified_towers = HashSet::<TetrahedronId>::new();
    let modified_towers_reference = Arc::new(Mutex::new(modified_towers));

    let modified_towers_update_lock = modified_towers_reference.clone();
    let modified_towers_reader_lock = modified_towers_reference.clone();

    let map_reader = map.clone();
    let map_updater = map.clone();


    // we keep track of which players have change in a hashset
    // we also save the changed players
    tokio::spawn(async move {
        loop {
            let message = rx_te_realtime_longterm.recv().await.unwrap();
            // println!("player entity changed  with inventory ? {}" , message.inventory.len());
            let mut modified_towers = modified_towers_update_lock.lock().await;
            modified_towers.insert(message.tetrahedron_id.clone());

            let mut towers_guard = map_updater.towers.lock().await;

            let old = towers_guard.get(&message.tetrahedron_id);
            match old {
                Some(_previous_record) => {
                    towers_guard.insert(message.tetrahedron_id.clone(), message);
                }
                _ => {
                   towers_guard.insert(message.tetrahedron_id.clone(), message);
                }
            }
            // we need to save into the hashmap and then save to a file.
        }
    });

    // after a few seconds we try to save all changes to the database.
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
            let mut modified_tower_keys = modified_towers_reader_lock.lock().await;
            let towers_guard = map_reader.towers.lock().await;

            let mut modified_tower_entities = Vec::<TowerEntity>::new();
            for id in modified_tower_keys.iter()
            {
                println!("this tower was changed {}", id.to_string());
                if let Some(tower_data) = towers_guard.get(id) 
                {
                    modified_tower_entities.push(tower_data.clone());
                }
            }

            modified_tower_keys.clear();
            drop(modified_tower_keys);
            drop(towers_guard);

            let data_collection: mongodb::Collection<StoredTower> = db_client.database("game").collection::<StoredTower>("towers");

            for tower in modified_tower_entities 
            {
                let updated_damage_record : Vec<StoredDamageByFaction> = tower.damage_received_in_event
                .into_iter()
                .map(|item| StoredDamageByFaction ::from(item))
                .collect();

                let serialized_data= bson::to_bson(&updated_damage_record).unwrap();

                let update_result = data_collection.update_one(
                    doc! {
                        "_id": tower.object_id,
                    },
                    doc! {
                        "$set": {
                            "damage_record" : serialized_data,
                            "faction": bson::to_bson(&tower.faction).unwrap(),
                            "event_id" :bson::to_bson(&tower.event_id).unwrap(),
                            "version" :bson::to_bson(&tower.version).unwrap(),
                        }
                    },
                    None
                ).await;

                println!("updated tower result {:?}", update_result);
            }
        }
    });
}



