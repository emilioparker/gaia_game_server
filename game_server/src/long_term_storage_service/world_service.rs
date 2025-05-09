
use std::collections::{HashSet, HashMap};
use std::io::Write;
use std::sync::Arc;
use std::time::SystemTime;
use crate::long_term_storage_service::db_region::StoredRegion;
use crate::long_term_storage_service::db_world::StoredWorld;
use crate::map::GameMap;
use crate::map::map_entity::{MapEntity};
use crate::map::tetrahedron_id::TetrahedronId;
use crate::{gaia_mpsc, ServerState};
use bson::doc;
use bson::oid::ObjectId;
use mongodb::Client;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use futures_util::stream::StreamExt;


// data is compressed
pub async fn preload_db(
    world_name : &str,
    world_id: Option<ObjectId>,
    regions_data : HashMap<TetrahedronId, Vec<u8>>,
    db_client : Client
) {

    let data_collection: mongodb::Collection<StoredRegion> = db_client.database("game").collection::<StoredRegion>("regions");
    let mut stored_regions = Vec::<StoredRegion>:: new();

    for region in regions_data
    {
        let bson = bson::Bson::Binary(bson::Binary 
        {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: region.1,
        });

        let data = StoredRegion {
            id : None,
            world_id : world_id,
            world_name : world_name.to_owned(),
            region_id : region.0.to_string(),
            region_version : 0,
            compressed_data : bson
        };

        stored_regions.push(data);
    }

    let insert_result = data_collection.insert_many(stored_regions, None).await.unwrap();
    cli_log::info!("{:?}", insert_result);

}

pub async fn get_regions_from_db(
    world_id : Option<ObjectId>,
    db_client : Client
) -> HashMap<TetrahedronId, StoredRegion> 
{

    let mut data = HashMap::<TetrahedronId, StoredRegion>::new();
    let data_collection: mongodb::Collection<StoredRegion> = db_client.database("game").collection::<StoredRegion>("regions");

    // Look up one document:
    let mut cursor = data_collection
    .find(
        doc! 
        {
                "world_id": world_id,
        },
        None,
    ).await
    .unwrap();

    let mut count = 0;
    while let Some(doc) = cursor.next().await {
        let region = doc.unwrap();
        // let binary_data: Vec<u8> = match region.compressed_data {
        //     bson::Bson::Binary(binary) => binary.bytes,
        //     _ => panic!("Expected Bson::Binary"),
        // };
        count += 1;
        data.insert( TetrahedronId::from_string(&region.region_id), region);
    }
    cli_log::info!("Got {} regions from database", count);

    data
}

pub async fn init_world_state( 
    world_name : &str,
    db_client : Client
) -> Option<ObjectId> 
{

    let data_collection: mongodb::Collection<StoredWorld> = db_client.database("game").collection::<StoredWorld>("worlds");

    let mut current_time = 0;
    let result = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    if let Ok(elapsed) = result 
    {
        current_time = elapsed.as_secs();
    }

    let data = StoredWorld 
    {
        id : None,
        world_name : world_name.to_owned(),
        start_time: current_time,
        inititalized: true,
        lod: 9,
    };

    let insert_result = data_collection.insert_one(data, None).await.unwrap();
    cli_log::info!("{:?}", insert_result);

    if let bson::Bson::ObjectId(id) = insert_result.inserted_id 
    {
        Some(id)
    }
    else 
    {
        None
    }
}


pub async fn check_world_state( 
    world_name : &str,
    db_client : Client
) -> Option<StoredWorld> 
{

    let data_collection: mongodb::Collection<StoredWorld> = db_client.database("game").collection::<StoredWorld>("worlds");

    // Look up one document:
    let data_from_db: Option<StoredWorld> = data_collection
    .find_one(
        doc! {
                "world_name": world_name.to_owned()
        },
        None,
    ).await
    .unwrap();
    cli_log::info!("stored_data: {:?}", data_from_db);
    data_from_db
}

pub fn start_server(
    mut rx_me_realtime_longterm : Receiver<MapEntity>,
    map : Arc<GameMap>,
    server_state : Arc<ServerState>,
    db_client : Client
) 
-> Receiver<u32>
{
    let (tx_saved_longterm_webservice, rx_me_tx_saved_longterm_webservice) = gaia_mpsc::channel::<u32>(100, crate::ServerChannels::TX_SAVED_LONGTERM_WEBSERVICE, server_state.clone());

    let modified_regions = HashSet::<TetrahedronId>::new();

    // used only for tracking
    let modified_tiles= HashSet::<TetrahedronId>::new();
    let modified_regions_reference = Arc::new(Mutex::new(modified_regions));

    let modified_regions_update_lock = modified_regions_reference.clone();
    let modified_regions_reader_lock = modified_regions_reference.clone();

    let map_reader = map.clone();
    let map_updater = map.clone();

    let map_reader_server_state = server_state.clone();
    let map_updater_server_state = server_state.clone();

    // we keep track of what tiles have change in a hashset
    // we also save the changed tiles in the gamemap.
    tokio::spawn(async move 
    {
        loop 
        {
            let message = rx_me_realtime_longterm.recv().await.unwrap();
            cli_log::info!("---got a tile changed {:?} ", message.id);
            let region_id = message.id.get_parent(7);

            let mut modified_regions = modified_regions_update_lock.lock().await;
            modified_regions.insert(region_id.clone());

            let region = map_updater.get_region(&region_id);
            let mut locked_tiles = region.lock().await;

            let old = locked_tiles.get(&message.id);
            match old 
            {
                Some(_previous_record) => 
                {
                    locked_tiles.insert(message.id.clone(), message);
                }
                _ => 
                {
                    locked_tiles.insert(message.id.clone(), message);
                }
            }

            map_updater_server_state.pending_regions_to_save.store(modified_regions.len() as u32, std::sync::atomic::Ordering::Relaxed);
        }
    });

    // after a few seconds we try to save all changes to the database.
    tokio::spawn(async move 
    {
        // init
        let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
        let current_time_in_millis = current_time.as_millis() as u64;
        map_reader_server_state.last_regions_save_timestamp.store(current_time_in_millis, std::sync::atomic::Ordering::Relaxed);

        loop 
        {
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await; // every 5 minutes
            let mut modified_regions = modified_regions_reader_lock.lock().await;

            let mut stored_regions = Vec::<StoredRegion>:: new();
            for region_id in modified_regions.iter()
            {
                cli_log::info!("this region was changed {}", region_id.to_string());

                let region = map_reader.get_region(region_id);
                let locked_tiles = region.lock().await;

                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));
                let mut region_object_id : Option<ObjectId> = None;

                for tile in locked_tiles.iter()
                {
                    region_object_id = tile.1.object_id;
                    let bytes = tile.1.to_bytes();
                    encoder.write_all(&bytes).unwrap();
                }

                let compressed_bytes = encoder.reset(Vec::new()).unwrap();
                let bson = bson::Bson::Binary(bson::Binary {
                    subtype: bson::spec::BinarySubtype::Generic,
                    bytes: compressed_bytes,
                });

                let data = StoredRegion {
                    id : region_object_id,
                    world_id : None,
                    world_name : "".to_owned(),
                    region_id : region_id.to_string(),
                    region_version : 0,
                    compressed_data : bson
                };

                stored_regions.push(data);
            }

            if modified_regions.len() > 0 
            {
                let data_collection: mongodb::Collection<StoredRegion> = db_client.database("game").collection::<StoredRegion>("regions");

                for region in stored_regions 
                {
                    // Update the document:
                    let update_result = data_collection.update_one(
                        doc! {
                            "world_id" : map_reader.world_id,
                            "_id": region.id,
                        },
                    doc! {
                            "$set": {"compressed_data": region.compressed_data},
                            "$inc": {"region_version": 1}
                        },
                        None
                    ).await;

                    cli_log::info!("updated region result {:?}", update_result);
                }
            }
            let _result =  tx_saved_longterm_webservice.send(1).await;

            modified_regions.clear();

            let pending = map_reader_server_state.pending_regions_to_save.load(std::sync::atomic::Ordering::Relaxed);
            map_reader_server_state.saved_regions.fetch_add(pending, std::sync::atomic::Ordering::Relaxed);
            map_reader_server_state.pending_regions_to_save.store(0, std::sync::atomic::Ordering::Relaxed);

            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            map_reader_server_state.last_regions_save_timestamp.store(current_time_in_millis, std::sync::atomic::Ordering::Relaxed);
        }
    });
    rx_me_tx_saved_longterm_webservice
}
