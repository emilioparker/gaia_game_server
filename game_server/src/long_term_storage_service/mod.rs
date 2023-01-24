
use std::collections::{HashSet, HashMap};
use std::io::Write;
use std::sync::Arc;
use std::time::SystemTime;
use crate::long_term_storage_service::db_world::StoredWorld;
use crate::map::GameMap;
use crate::map::map_entity::{MapEntity};
use crate::map::tetrahedron_id::TetrahedronId;
use bson::doc;
use bson::oid::ObjectId;
use mongodb::Client;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use futures_util::stream::StreamExt;


use self::db_region::StoredRegion;

pub mod db_region;
pub mod db_world;

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
) -> HashMap<TetrahedronId, Vec<u8>> {

    let mut data = HashMap::<TetrahedronId, Vec<u8>>::new();

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
        let binary_data: Vec<u8> = match region.compressed_data {
            bson::Bson::Binary(binary) => binary.bytes,
            _ => panic!("Expected Bson::Binary"),
        };
        count += 1;
        data.insert( TetrahedronId::from_string(&region.region_id), binary_data);
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
    mut tile_changes_rx : Receiver<MapEntity>,
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


    tokio::spawn(async move {
        loop {
            let message = tile_changes_rx.recv().await.unwrap();
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

    tokio::spawn(async move {
        loop {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));
            tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
            let mut modified_regions = modified_regions_reader_lock.lock().await;

            for region_id in modified_regions.iter(){
                println!("this region was changed {}", region_id.to_string());
                let region = map_reader.get_region(region_id);
                let file_name = format!("map_working_data/world_002_{}_props.bytes", region_id.to_string());
                let mut file = File::create(file_name).await.unwrap();

                let locked_tiles = region.lock().await;
                for tile in locked_tiles.iter()
                {
                    let bytes = tile.1.to_bytes();
                    encoder.write(&bytes).unwrap();
                }

                let compressed_bytes = encoder.reset(Vec::new()).unwrap();
                file.write_all(&compressed_bytes).await.unwrap();

            }
            modified_regions.clear();
        }
    });
}

#[cfg(test)]
mod tests {
    use std::env;
    use bson::{oid::ObjectId, document};
    use mongodb::{Client, options::{ClientOptions, ResolverConfig}};
    use chrono::{TimeZone, Utc};
    use mongodb::bson::doc;
    use serde::{Serialize, Deserialize};

    #[test]
    fn test_doc() {

        let new_doc = doc! {
        "title": "Parasite",
        "year": 2020,
        "plot": "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.",
        "released": Utc.ymd(2020, 2, 7).and_hms_opt(0, 0, 0),
        };

        println!("{}", new_doc);
    }

    // fn insert_test()
    // {
    //     let new_doc = doc! {
    //     "title": "Parasite",
    //     "year": 2020,
    //     "plot": "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.",
    //     "released": Utc.ymd(2020, 2, 7).and_hms_opt(0, 0, 0),
    //     };

    //     println!("{}", new_doc);
    //     let insert_result = movies.insert_one(new_doc.clone(), None).await?;
    //     println!("New document ID: {}", insert_result.inserted_id);
    // }

    #[tokio::test]
    async fn test_something_async() {
        let new_doc = doc! {
            "title": "Parasite",
            "year": 2020,
            "plot": "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.",
            "released": Utc.with_ymd_and_hms(2020, 2, 7, 0,0,0).unwrap()
        };
        
        // let client_uri = env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
        let client_uri = "mongodb://localhost:27017/test?retryWrites=true&w=majority";


        // A Client is needed to connect to MongoDB:
        // An extra line of code to work around a DNS issue on Windows:
        let options = ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare()).await.unwrap();
        let client = Client::with_options(options).unwrap();
        // Print the databases in our MongoDB cluster:
        println!("Databases:");
        for name in client.list_database_names(None, None).await.unwrap() {
            println!("- {}", name);
        }
    }

    #[tokio::test]
    async fn test_insert() {
        let new_doc = doc! {
            "title": "Parasite",
            "year": 2020,
            "plot": "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.",
            "released": Utc.with_ymd_and_hms(2020, 2, 7, 0,0,0).unwrap()
        };
        
        // let client_uri = env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
        let client_uri = "mongodb://localhost:27017/test?retryWrites=true&w=majority";


        // A Client is needed to connect to MongoDB:
        // An extra line of code to work around a DNS issue on Windows:
        let options = ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare()).await.unwrap();
        let client = Client::with_options(options).unwrap();
        // Print the databases in our MongoDB cluster:
        println!("Databases:");
        for name in client.list_database_names(None, None).await.unwrap() {
            println!("- {}", name);
        }
        let movies = client.database("sample_mflix").collection("movies");

        let new_doc = doc! {
        "title": "Parasite",
        "year": 2021,
        "plot": "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.",
        };

        println!("{}", new_doc);
        let insert_result = movies.insert_one(new_doc.clone(), None).await.unwrap();
        println!("New document ID: {}", insert_result.inserted_id);
    }

    // You use `serde` to create structs which can serialize & deserialize between BSON:
    #[derive(Serialize, Deserialize, Debug)]
    struct Data {
        #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
        id: Option<ObjectId>,
        title: String,
        year: i32,
        plot: String,
        compressed_data : Vec<u8>,
    }


    #[tokio::test]
    async fn test_insert_struct() {
        let new_doc = doc! {
            "title": "Parasite",
            "year": 2020,
            "plot": "A poor family, the Kims, con their way into becoming the servants of a rich family, the Parks. But their easy life gets complicated when their deception is threatened with exposure.",
            "released": Utc.with_ymd_and_hms(2020, 2, 7, 0,0,0).unwrap()
        };
        
        // let client_uri = env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
        let client_uri = "mongodb://localhost:27017/test?retryWrites=true&w=majority";

        // A Client is needed to connect to MongoDB:
        // An extra line of code to work around a DNS issue on Windows:
        let options = ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare()).await.unwrap();
        let client = Client::with_options(options).unwrap();
        // Print the databases in our MongoDB cluster:
        println!("Databases:");
        for name in client.list_database_names(None, None).await.unwrap() {
            println!("- {}", name);
        }

        let data_collection: mongodb::Collection<bson::Document> = client.database("game").collection("main_data");

        let bin = vec![1, 2, 3, 4, 5];
// let binary_data = Bson::Binary(bin);

// let insert_doc = doc! { "binary_field": binary_data };

        let data = Data {
            id : None,
            title : "A".to_owned(),
            year : 2,
            plot : "something boring".to_owned(),
            compressed_data : bin
        };
        let serialized_data= bson::to_bson(&data).unwrap();
        let document = serialized_data.as_document().unwrap();

        let insert_result = data_collection.insert_one(document.to_owned(), None).await.unwrap();

        println!("New document ID: {}", insert_result.inserted_id);

    }
}



