
use std::collections::{HashSet, HashMap};
use std::io::Write;
use std::sync::Arc;
use std::time::SystemTime;
use crate::long_term_storage_service::db_region::StoredRegion;
use crate::long_term_storage_service::db_world::StoredWorld;
use crate::map::GameMap;
use crate::map::map_entity::{MapEntity};
use crate::map::tetrahedron_id::TetrahedronId;
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
        let bson = bson::Bson::Binary(bson::Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: region.1,
        });

        let data = StoredRegion {
            id : None,
            world_id : world_id,
            world_name : world_name.to_owned(),
            region_id : region.0.to_string(),
            compressed_data : bson
        };

        stored_regions.push(data);
    }

    let insert_result = data_collection.insert_many(stored_regions, None).await.unwrap();
    println!("{:?}", insert_result);

}

pub async fn get_regions_from_db(
    world_name : &str,
    db_client : Client
) -> HashMap<TetrahedronId, StoredRegion> {

    let mut data = HashMap::<TetrahedronId, StoredRegion>::new();

    let data_collection: mongodb::Collection<StoredRegion> = db_client.database("game").collection::<StoredRegion>("regions");

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
    while let Some(doc) = cursor.next().await {
        let region = doc.unwrap();
        // let binary_data: Vec<u8> = match region.compressed_data {
        //     bson::Bson::Binary(binary) => binary.bytes,
        //     _ => panic!("Expected Bson::Binary"),
        // };
        count += 1;
        data.insert( TetrahedronId::from_string(&region.region_id), region);
    }
    println!("Got {} regions from database", count);

    data
}

pub async fn init_world_state( 
    world_name : &str,
    db_client : Client
) -> Option<ObjectId> {

    let data_collection: mongodb::Collection<StoredWorld> = db_client.database("game").collection::<StoredWorld>("worlds");

    let mut current_time = 0;
    let result = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    if let Ok(elapsed) = result {
        current_time = elapsed.as_secs();
    }

    let data = StoredWorld {
        id : None,
        world_name : world_name.to_owned(),
        start_time: current_time,
        inititalized: true,
        lod: 9,
    };

    let insert_result = data_collection.insert_one(data, None).await.unwrap();
    println!("{:?}", insert_result);

    if let bson::Bson::ObjectId(id) = insert_result.inserted_id {
        Some(id)
    }
    else {
        None
    }
}


pub async fn check_world_state( 
    world_name : &str,
    db_client : Client
) -> Option<StoredWorld> {

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
    println!("stored_data: {:?}", data_from_db);
    data_from_db
}

pub fn start_server(
    mut rx_me_realtime_longterm : Receiver<MapEntity>,
    map : GameMap,
    db_client : Client
) {
    let modified_regions = HashSet::<TetrahedronId>::new();
    let modified_regions_reference = Arc::new(Mutex::new(modified_regions));

    let modified_regions_update_lock = modified_regions_reference.clone();
    let modified_regions_reader_lock = modified_regions_reference.clone();

    let map_reference = Arc::new(map);
    let map_reader = map_reference.clone();
    let map_updater = map_reference.clone();


    // we keep track of what tiles have change in a hashset
    // we also save the changed tiles in the gamemap.
    tokio::spawn(async move {
        loop {
            let message = rx_me_realtime_longterm.recv().await.unwrap();
            println!("got a tile changed {:?} ", message);
            let region_id = message.id.get_parent(7);

            let mut modified_regions = modified_regions_update_lock.lock().await;
            modified_regions.insert(region_id.clone());

            let region = map_updater.get_region(&region_id);
            let mut locked_tiles = region.lock().await;

            let old = locked_tiles.get(&message.id);
            match old {
                Some(_previous_record) => {
                    locked_tiles.insert(message.id.clone(), message);
                }
                _ => {
                   locked_tiles.insert(message.id.clone(), message);
                }
            }
            // we need to save into the hashmap and then save to a file.
        }
    });

    // after a few seconds we try to save all changes to the database.
    tokio::spawn(async move {
        loop {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));
            tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
            let mut modified_regions = modified_regions_reader_lock.lock().await;

            let mut stored_regions = Vec::<StoredRegion>:: new();
            for region_id in modified_regions.iter(){
                println!("this region was changed {}", region_id.to_string());

                let region = map_reader.get_region(region_id);

                let locked_tiles = region.lock().await;
                let mut region_object_id : Option<ObjectId> = None;

                for tile in locked_tiles.iter()
                {
                    region_object_id = tile.1.object_id;
                    let bytes = tile.1.to_bytes();
                    encoder.write(&bytes).unwrap();
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
                    compressed_data : bson
                };

                stored_regions.push(data);
            }
            modified_regions.clear();

            if modified_regions.len() > 0 {
                let data_collection: mongodb::Collection<StoredRegion> = db_client.database("game").collection::<StoredRegion>("regions");

                for region in stored_regions {
                    // Update the document:
                    let update_result = data_collection.update_one(
                        doc! {
                            "_id": region.id,
                        },
                    doc! {
                            "$set": {"compressed_data": region.compressed_data}
                        },
                        None
                    ).await;

                    println!("updated region result {:?}", update_result);
                }
            }
        }
    });
}
