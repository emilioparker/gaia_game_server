
use std::collections::{HashSet, HashMap};
use std::sync::Arc;
use crate::kingdom::kingdom_entity::KingdomEntity;
use crate::long_term_storage_service::db_kingdom::StoredKingdom;
use crate::{gaia_mpsc, get_faction_code, get_faction_from_code, ServerState};
use crate::long_term_storage_service::db_tower::StoredDamageByFaction;
use crate::map::GameMap;
use crate::map::tetrahedron_id::TetrahedronId;
use bson::doc;
use bson::oid::ObjectId;
use mongodb::Client;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use futures_util::stream::StreamExt;



pub async fn get_kingdoms_from_db_by_world(
    world_id : Option<ObjectId>,
    db_client : Client
) 
-> HashMap<TetrahedronId, KingdomEntity>
{
    cli_log::info!("get kingdoms from db using {:?}", world_id);

    let mut data = HashMap::<TetrahedronId, KingdomEntity>::new();

    let data_collection: mongodb::Collection<StoredKingdom> = db_client.database("game").collection::<StoredKingdom>("kingdoms");

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
                let kingdom_entity =  KingdomEntity
                {
                    object_id: doc.id,
                    version:doc.version,
                    tetrahedron_id: TetrahedronId::from_string(&doc.tetrahedron_id),
                    faction:get_faction_code(&doc.faction),
                };

                // cli_log::info!("-------Add tower {}", tower.tetrahedron_id);
                count += 1;
                data.insert(kingdom_entity.tetrahedron_id.clone(), kingdom_entity);
            },
            Err(error_details) => 
            {
                cli_log::info!("error getting kingdoms from db with {:?}", error_details);
            },
        }
    }
    cli_log::info!("Got {} kingdoms from database", count);

    data
}

pub async fn preload_db(
    world_name : &str,
    world_id: Option<ObjectId>,
    kingdoms : Vec<(TetrahedronId, u8)>,
    db_client : Client
) {

    let data_collection: mongodb::Collection<StoredKingdom> = db_client.database("game").collection::<StoredKingdom>("kingdoms");
    let mut stored_kingdoms = Vec::<StoredKingdom>:: new();

    for kingdom_id in kingdoms
    {
        let data = StoredKingdom 
        {
            id : None,
            tetrahedron_id : kingdom_id.0.to_string(),
            world_id : world_id,
            world_name : world_name.to_owned(),
            version : 0,
            faction : get_faction_from_code(kingdom_id.1),
        };

        stored_kingdoms.push(data);
    }

    let insert_result = data_collection.insert_many(stored_kingdoms, None).await.unwrap();
    cli_log::info!("{:?}", insert_result);

}
pub fn start_server(
    mut rx_ke_realtime_longterm : Receiver<KingdomEntity>,
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    db_client : Client)
    -> Receiver<bool>
    {

    let (tx_ke_saved_longterm_webservice, rx_ke_saved_longterm_webservice) = gaia_mpsc::channel::<bool>(100, crate::ServerChannels::TX_KE_SAVED_LONGTERM_WEBSERVICE, server_state.clone());

    let modified_kingdomes = HashSet::<TetrahedronId>::new();
    let modified_kingdomes_reference = Arc::new(Mutex::new(modified_kingdomes));

    let modified_kingdomes_update_lock = modified_kingdomes_reference.clone();
    let modified_kingdomes_reader_lock = modified_kingdomes_reference.clone();

    let map_reader = map.clone();
    let map_updater = map.clone();

    let map_reader_server_state = server_state.clone();
    let map_updater_server_state = server_state.clone();

    // we keep track of which kingdomes have change in a hashset
    // we also save the changed kingdomes
    tokio::spawn(async move 
    {
        loop 
        {
            let message = rx_ke_realtime_longterm.recv().await.unwrap();
            // cli_log::info!("player entity changed  with inventory ? {}" , message.inventory.len());
            let mut modified_kingdomes = modified_kingdomes_update_lock.lock().await;
            modified_kingdomes.insert(message.tetrahedron_id.clone());

            let mut kingdomes_guard = map_updater.kingdomes.lock().await;

            let old = kingdomes_guard.get(&message.tetrahedron_id);
            match old 
            {
                Some(_previous_record) => 
                {
                    kingdomes_guard.insert(message.tetrahedron_id.clone(), message);
                }
                _ => 
                {
                   kingdomes_guard.insert(message.tetrahedron_id.clone(), message);
                }
            }
            // we need to save into the hashmap and then save to a file.

            map_updater_server_state.pending_kingdome_entities_to_save.store(kingdomes_guard.len() as u32, std::sync::atomic::Ordering::Relaxed);
        }
    });

    // after a few seconds we try to save all changes to the database.
    tokio::spawn(async move 
    {
        // init
        let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
        let current_time_in_millis = current_time.as_millis() as u64;
        map_reader_server_state.last_kingdome_entities_save_timestamp.store(current_time_in_millis, std::sync::atomic::Ordering::Relaxed);

        loop 
        {
            tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
            let mut modified_kingdome_keys = modified_kingdomes_reader_lock.lock().await;
            let modified_kingdomes= modified_kingdome_keys.len();
            let kingdomes_guard = map_reader.kingdomes.lock().await;

            let mut modified_kingdome_entities = Vec::<KingdomEntity>::new();
            for id in modified_kingdome_keys.iter()
            {
                cli_log::info!("this kingdome has changed {}", id.to_string());
                if let Some(kingdome_data) = kingdomes_guard.get(id) 
                {
                    modified_kingdome_entities.push(kingdome_data.clone());
                }
            }

            modified_kingdome_keys.clear();
            drop(modified_kingdome_keys);
            drop(kingdomes_guard);

            let data_collection: mongodb::Collection<StoredKingdom> = db_client.database("game").collection::<StoredKingdom>("kingdoms");

            for kingdome in modified_kingdome_entities 
            {
                let update_result = data_collection.update_one(
                    doc! 
                    {
                        "_id": kingdome.object_id,
                    },
                    doc! 
                    {
                        "$set": 
                        {
                            "faction": get_faction_from_code(kingdome.faction),
                            "version" :bson::to_bson(&kingdome.version).unwrap(),
                        }
                    },
                    None
                ).await;


                cli_log::info!("updated kingdom result {:?}", update_result);
            }

            map_reader_server_state.pending_kingdome_entities_to_save.store(0, std::sync::atomic::Ordering::Relaxed);

            // let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            // let current_time_in_millis = current_time.as_millis() as u64;
            // map_reader_server_state.last_tower_entities_save_timestamp.store(current_time_in_millis, std::sync::atomic::Ordering::Relaxed);

            let _result = tx_ke_saved_longterm_webservice.send(true).await;
        }
    });
    rx_ke_saved_longterm_webservice
}



