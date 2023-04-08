use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use flate2::read::ZlibDecoder;
use game_server::ServerState;
use game_server::gameplay_service;
use game_server::long_term_storage_service;
use game_server::long_term_storage_service::db_region::StoredRegion;
use game_server::map::GameMap;
use game_server::map::map_entity::MAP_ENTITY_SIZE;
use game_server::map::map_entity::MapEntity;
use game_server::map::tetrahedron_id::TetrahedronId;
use game_server::real_time_service;
use game_server::web_service;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use mongodb::Client;
use mongodb::options::ClientOptions;
use mongodb::options::ResolverConfig;


// #[tokio::main(worker_threads = 1)]
#[tokio::main()]
async fn main() {

    let mut main_loop = tokio::time::interval(std::time::Duration::from_millis(50000));

    let server_state = Arc::new(ServerState{
        tx_mc_client_gameplay: AtomicUsize::new(0),
        tx_pc_client_gameplay: AtomicUsize::new(0),
        tx_bytes_gameplay_socket: AtomicUsize::new(0),
        tx_me_gameplay_longterm:AtomicUsize::new(0),
        tx_me_gameplay_webservice:AtomicUsize::new(0),
        tx_pe_gameplay_longterm:AtomicUsize::new(0)
    });
    // let (_tx, mut rx) = tokio::sync::watch::channel("hello");

    let client_uri = "mongodb://localhost:27017/test?retryWrites=true&w=majority";
    let options = ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare()).await.unwrap();
    let db_client = Client::with_options(options).unwrap();

    let world_name = "world_019";

    let working_game_map: Option<GameMap>; // load_files_into_game_map(world_name).await;
    let storage_game_map: Option<GameMap>; // load_files_into_game_map(world_name).await;

    let world_state = long_term_storage_service::world_service::check_world_state(world_name, db_client.clone()).await;


    //shared by the realtime service and the webservice

    if let Some(world) = world_state {
        println!("Load the world from db init at {}", world.start_time);
        let working_players = long_term_storage_service::players_service::get_players_from_db(world.id, db_client.clone()).await;
        //used and updated by the long storage system
        let storage_players = working_players.clone();

        let regions_db_data = long_term_storage_service::world_service::get_regions_from_db(world.id, db_client.clone()).await;
        println!("reading regions into game maps");
        let regions_data = load_regions_data_into_game_map(&regions_db_data);
        working_game_map = Some(GameMap::new(world.id, regions_data.clone(), working_players));
        storage_game_map = Some(GameMap::new(world.id, regions_data, storage_players));
    }
    else{
        println!("Creating world from scratch, because it was not found in the database");
        // any errors will just crash the app.

        let world_id = long_term_storage_service::world_service::init_world_state(world_name, db_client.clone()).await;
        if let Some(id) = world_id{
            println!("Creating world with id {}", id);
            let working_players = long_term_storage_service::players_service::get_players_from_db(world_id, db_client.clone()).await;
            //used and updated by the long storage system
            let storage_players = working_players.clone();

            let regions_data = load_files_into_regions_hashset(world_name).await;
            long_term_storage_service::world_service::preload_db(world_name, world_id, regions_data, db_client.clone()).await;

            // reading what we just created because we need the object ids!
            let regions_data_from_db = long_term_storage_service::world_service::get_regions_from_db(world_id, db_client.clone()).await;

            let regions_data = load_regions_data_into_game_map(&regions_data_from_db);
            working_game_map = Some(GameMap::new(world_id, regions_data.clone(), working_players));
            storage_game_map = Some(GameMap::new(world_id, regions_data, storage_players));
        }
        else {
            println!("Error creating world in db");
            return;
        }
    }


    match (working_game_map, storage_game_map) {
        (Some(working_game_map), Some(storage_game_map)) =>
        {
            let working_game_map_reference= Arc::new(working_game_map);
            let storage_game_map_reference= Arc::new(storage_game_map);


            let (rx_mc_client_gameplay,
                rx_pc_client_gameplay, 
                tx_bytes_gameplay_socket 
            ) =  real_time_service::start_server(
                working_game_map_reference.clone(), 
                server_state.clone());

            let (rx_me_gameplay_longterm,
                rx_me_gameplay_webservice,
                rx_pe_gameplay_longterm,
                tx_mc_webservice_gameplay,
            ) = gameplay_service::start_service(
                rx_pc_client_gameplay,
                rx_mc_client_gameplay,
                working_game_map_reference.clone(), 
                server_state.clone(),
                tx_bytes_gameplay_socket);

            // realtime service sends the mapentity after updating the working copy, so it can be stored eventually
            let rx_saved_longterm_web_service = long_term_storage_service::world_service::start_server(
                rx_me_gameplay_longterm,
                storage_game_map_reference.clone(), 
                db_client.clone()
            );

            long_term_storage_service::players_service::start_server(
                rx_pe_gameplay_longterm,
                storage_game_map_reference.clone(), 
                db_client.clone()
            );
            
            web_service::start_server(
                working_game_map_reference, 
                storage_game_map_reference, 
                db_client.clone(),
                rx_me_gameplay_webservice,
                tx_mc_webservice_gameplay,
                rx_saved_longterm_web_service,
            );
        // ---------------------------------------------------
        },
        _ => {
            println!("big and horrible error with the working and storage tiles");
        }
    }

    println!("Game server started correctly");
    loop {
        // assuming 30 fps.
        // tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        main_loop.tick().await;
        // println!("{:?}", server_state);
    }
}




fn load_regions_data_into_game_map(
    regions_stored_data : &HashMap<TetrahedronId, StoredRegion>
) -> Vec<(TetrahedronId, HashMap<TetrahedronId, MapEntity>)> {

    let mut regions_data = Vec::<(TetrahedronId, HashMap<TetrahedronId, MapEntity>)>::new();

    let mut count = 0;
    let mut region_count = 0;
    let region_total = regions_stored_data.len();

    for region in regions_stored_data.iter(){
        region_count += 1;
        println!("decoding region progress {region_count}/{region_total} tiles {count}");

        let region_object_id = region.1.id.clone();
        let binary_data: Vec<u8> = match region.1.compressed_data.clone() {
            bson::Bson::Binary(binary) => binary.bytes,
            _ => panic!("Expected Bson::Binary"),
        };
        let region_id = region.0;
        let data : &[u8] = &binary_data;
        let decoder = ZlibDecoder::new(data);

        let decoded_data_result :  Result<Vec<u8>, _> = decoder.bytes().collect();
        let decoded_data = decoded_data_result.unwrap();
        let tiles : &[u8] = &decoded_data;
        let size = tiles.len();

        let mut buffer = [0u8;MAP_ENTITY_SIZE as usize];
        let mut start = 0;
        let mut end = MapEntity::get_size() as usize;

        // println!("initialy for region {} {}",region_id, all_tiles.len());

        let mut region_tiles : HashMap<TetrahedronId, MapEntity> = HashMap::new();

        loop {
            buffer.copy_from_slice(&tiles[start..end]);
            let mut map_entity = MapEntity::from_bytes(&buffer);
            // all map entities will have the object id of the database region, this value is the same for all map entities in a region
            map_entity.object_id = region_object_id;
            
            if map_entity.id.to_string() == "j202020303" {
                println!("Found saved entity  {:?} " , map_entity);
            }
            region_tiles.insert(map_entity.id.clone(), map_entity);


            start = end;
            end = end + MapEntity::get_size();

            if end > size
            {
                break;
            }
            // counting mapentities
            count += 1;

        }
        regions_data.push((region_id.clone(), region_tiles));
    }

    println!("finished loading data, starting services. regions: {} with {} tiles",region_total, count);
    regions_data
    // GameMap::new(regions_data)
}

async fn get_compressed_tiles_data_from_file(world_id : &str, region_id : String) -> Vec<u8> {
    let file_name = format!("map_initial_data/{}_{}_props.bytes",world_id, region_id);
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));
    println!("reading file {}", file_name);

    let tiles = tokio::fs::read(file_name).await.unwrap();
    let size = tiles.len();

    let mut buffer = [0u8;MAP_ENTITY_SIZE];
    let mut start = 0;
    let mut end = MapEntity::get_size();

    loop {
        buffer.copy_from_slice(&tiles[start..end]);
        encoder.write_all(&buffer).unwrap();

        start = end;
        end = end + MapEntity::get_size();
        if end > size
        {
            break;
        }
    }

    let compressed_bytes = encoder.reset(Vec::new()).unwrap();
    compressed_bytes
}

async fn load_files_into_regions_hashset(world_id : &str) -> HashMap<TetrahedronId, Vec<u8>> {

    let regions = game_server::map::get_region_ids(2);
    let mut regions_data = HashMap::<TetrahedronId, Vec<u8>>::new();
    for region in regions
    {
        let data = get_compressed_tiles_data_from_file(world_id, region.to_string()).await;
        regions_data.insert(region, data);
    }
    regions_data
}


#[cfg(test)]
mod tests {
    use std::{env, io::Write, collections::HashMap};
    use bson::{oid::ObjectId, document};
    use game_server::{long_term_storage_service::{self, db_region::StoredRegion}, map::{GameMap, tetrahedron_id::{self, TetrahedronId}, map_entity::MapEntity}};
    use mongodb::{Client, options::{ClientOptions, ResolverConfig}};
    use chrono::{TimeZone, Utc};
    use mongodb::bson::doc;
    use serde::{Serialize, Deserialize};
    use flate2::{write::ZlibEncoder, Compression};

    use crate::load_regions_data_into_game_map;

    #[tokio::test]
    async fn test_insert() {
        let world_name = "test_world_015";
        let client_uri = "mongodb://localhost:27017/test?retryWrites=true&w=majority";
        let options = ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare()).await.unwrap();
        let db_client = Client::with_options(options).unwrap();
        let data_collection: mongodb::Collection<StoredRegion> = db_client.database("game").collection::<StoredRegion>("regions");



        // let world_state = long_term_storage_service::world_service::check_world_state(world_name, db_client.clone()).await;

// reading the data
        // let world = world_state.unwrap(); 
        // let working_players = long_term_storage_service::players_service::get_players_from_db(world.id, db_client.clone()).await;
        // let regions_db_data = long_term_storage_service::world_service::get_regions_from_db(world.id, db_client.clone()).await;
        // println!("reading regions into game maps");
        // let regions_data = load_regions_data_into_game_map(&regions_db_data);

        // let working_game_map = GameMap::new(world.id, regions_data, working_players);
// manipulating the data a bit.
        let tile_id = TetrahedronId::from_string("j202020303");
        let tile_id_1= TetrahedronId::from_string("j202020302");
        let tile_id_2= TetrahedronId::from_string("j202020301");
        let region_id = tile_id.get_parent(7);

        let mut region = HashMap::new();
        region.insert(tile_id.clone(), MapEntity::new("j202020303", 100));
        region.insert(tile_id_1.clone(), MapEntity::new("j202020302", 101));
        region.insert(tile_id_2.clone(), MapEntity::new("j202020301", 102));

        let delete_result = data_collection.delete_one(doc! {
            "world_name": world_name.to_string(),
            "region_id": region_id.to_string()
        }, None).await;

        println!("delete result {delete_result:?}");

        // let mut locked_tiles = region.lock().await;

        let old = region.get(&tile_id);
        match old {
            Some(previous_record) => {
                println!("got a {:?}", previous_record);
                let new_tile = MapEntity{
                    health: 50,
                    ..previous_record.clone()
                };
                region.insert(tile_id.clone(), new_tile);
            }
            _ => {
                println!("not found");
                assert!(false);
                // locked_tiles.insert(message.id.clone(), message);
            }
        }
        // checking if update worked.

        let old = region.get(&tile_id);
        match old {
            Some(previous_record) => {
                println!("got a {:?}", previous_record);
                assert!(previous_record.health == 50);
            }
            _ => {
                println!("not found");
                assert!(false);
                // locked_tiles.insert(message.id.clone(), message);
            }
        }

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));

        // let mut region_object_id : Option<ObjectId> = None;
        for tile in region.iter()
        {
            let bytes = tile.1.to_bytes();
            encoder.write_all(&bytes).unwrap();
        }

        let compressed_bytes = encoder.reset(Vec::new()).unwrap();
        let bson = bson::Bson::Binary(bson::Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: compressed_bytes,
        });

        let data = StoredRegion {
            id : None,
            world_id : None,
            world_name : world_name.to_string(),
            region_id : region_id.to_string(),
            region_version : 0,
            compressed_data : bson
        };
    

        let insert_result = data_collection.insert_one(data, None).await;

        println!("update_result {insert_result:?}");

        let mut recovered_region = data_collection
        .find_one(
            doc! {
                    "world_name": world_name.to_string(),
                    "region_id": region_id.to_string()
            },
            None,
        ).await
        .unwrap()
        .unwrap();

        let mut map = HashMap::new();
        map.insert(region_id.clone(), recovered_region);

        let regions_data = load_regions_data_into_game_map(&map);
        let decoded_region = &regions_data[0];
        let tile = decoded_region.1.get(&tile_id).unwrap();
        assert!(tile.health == 50);

        let old = region.get(&tile_id);
        match old {
            Some(previous_record) => {
                println!("got a {:?}", previous_record);
                let new_tile = MapEntity{
                    health: 20,
                    ..previous_record.clone()
                };
                region.insert(tile_id.clone(), new_tile);
            }
            _ => {
                println!("not found");
                assert!(false);
                // locked_tiles.insert(message.id.clone(), message);
            }
        }

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));

        // let mut region_object_id : Option<ObjectId> = None;
        for tile in region.iter()
        {
            let bytes = tile.1.to_bytes();
            encoder.write_all(&bytes).unwrap();
        }

        let compressed_bytes = encoder.reset(Vec::new()).unwrap();
        let bson = bson::Bson::Binary(bson::Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: compressed_bytes,
        });

        let data = StoredRegion {
            id : None,
            world_id : None,
            world_name : world_name.to_string(),
            region_id : region_id.to_string(),
            region_version : 0,
            compressed_data : bson
        };


        let update_result = data_collection.update_one(
            doc! {
                "world_name" :world_name.to_string(),
                "region_id": region_id.to_string()
            },
            doc! {
                "$set": {"compressed_data": data.compressed_data}
            },
            None
        ).await;
        
        println!("update_result {update_result:?}");


        let recovered_region = data_collection
        .find_one(
            doc! {
                    "world_name": world_name.to_string(),
                    "region_id": region_id.to_string()
            },
            None,
        ).await
        .unwrap()
        .unwrap();

        let mut map = HashMap::new();
        map.insert(region_id.clone(), recovered_region);

        let regions_data = load_regions_data_into_game_map(&map);
        let decoded_region = &regions_data[0];
        let tile = decoded_region.1.get(&tile_id).unwrap();
        assert!( tile.health == 20);


    }
}
