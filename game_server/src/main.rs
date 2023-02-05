use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;

use flate2::read::ZlibDecoder;
use game_server::gameplay_service;
use game_server::long_term_storage_service;
use game_server::long_term_storage_service::db_region::StoredRegion;
use game_server::map::GameMap;
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

    let (_tx, mut rx) = tokio::sync::watch::channel("hello");

    let client_uri = "mongodb://localhost:27017/test?retryWrites=true&w=majority";
    let options = ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare()).await.unwrap();
    let db_client = Client::with_options(options).unwrap();

    // tiles are modified by many systems, but since we only have one core... our mutex doesn't work too much
    let world_name = "world_002";

    let working_game_map: Option<GameMap>; // load_files_into_game_map(world_name).await;
    let storage_game_map: Option<GameMap>; // load_files_into_game_map(world_name).await;

    let world_state = long_term_storage_service::world_service::check_world_state(world_name, db_client.clone()).await;

    let working_players = long_term_storage_service::players_service::get_players_from_db(world_name, db_client.clone()).await;
    //used and updated by the long storage system
    let storage_players = working_players.clone();

    //shared by the realtime service and the webservice

    if let Some(world) = world_state {
        println!("Load the world from db init at {}", world.start_time);
        let regions_db_data = long_term_storage_service::world_service::get_regions_from_db(world_name, db_client.clone()).await;
        println!("reading regions into game maps");
        let regions_data = load_regions_data_into_game_map(&regions_db_data);
        working_game_map = Some(GameMap::new(regions_data.clone(), working_players));
        storage_game_map = Some(GameMap::new(regions_data, storage_players));
    }
    else{
        println!("Creating world from scratch, because it was not found in the database");
        // any errors will just crash the app.

        let world_id = long_term_storage_service::world_service::init_world_state(world_name, db_client.clone()).await;
        if let Some(id) = world_id{
            println!("Creating world with id {}", id);
            let regions_data = load_files_into_regions_hashset(world_name).await;
            long_term_storage_service::world_service::preload_db(world_name, world_id, regions_data, db_client.clone()).await;

            // reading what we just created because we need the object ids!
            let regions_data_from_db = long_term_storage_service::world_service::get_regions_from_db(world_name, db_client.clone()).await;

            let regions_data = load_regions_data_into_game_map(&regions_data_from_db);
            working_game_map = Some(GameMap::new(regions_data.clone(), working_players));
            storage_game_map = Some(GameMap::new(regions_data, storage_players));
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

            let rx_mc_webservice_gameplay = web_service::start_server(
                working_game_map_reference.clone(), 
                db_client.clone()
            );

            let (rx_mc_client_gameplay,
                rx_pa_client_gameplay, 
                tx_bytes_gameplay_socket 
            ) =  real_time_service::start_server();

            let (rx_me_gameplay_longterm,
                rx_pe_gameplay_longterm
            ) = gameplay_service::start_service(
                rx_pa_client_gameplay,
                rx_mc_client_gameplay,
                rx_mc_webservice_gameplay,
                working_game_map_reference.clone(), 
                tx_bytes_gameplay_socket);

            // realtime service sends the mapentity after updating the working copy, so it can be stored eventually
            long_term_storage_service::world_service::start_server(
                rx_me_gameplay_longterm,
                storage_game_map_reference.clone(), 
                db_client.clone()
            );
            long_term_storage_service::players_service::start_server(
                rx_pe_gameplay_longterm,
                storage_game_map_reference, 
                db_client.clone()
            );
        // ---------------------------------------------------
        },
        _ => {
            println!("big and horrible error with the working and storage tiles");
        }
    }

    println!("Game server started correctly");
    rx.changed().await.unwrap();
}


fn get_regions(initial : TetrahedronId, target_lod : u8, regions : &mut Vec<TetrahedronId>)
{
    if initial.lod == target_lod
    {
        regions.push(initial);
    }
    else {
        for index in 0..4
        {
            get_regions(initial.subdivide(index), target_lod, regions);
        }
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
        println!("decoding region {} progress {}/{}", region.0, region_count, region_total);
        region_count += 1;
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

        let mut buffer = [0u8;69];
        let mut start = 0;
        let mut end = 69;

        // println!("initialy for region {} {}",region_id, all_tiles.len());

        let mut region_tiles : HashMap<TetrahedronId, MapEntity> = HashMap::new();

        loop {
            buffer.copy_from_slice(&tiles[start..end]);
            let mut map_entity = MapEntity::from_bytes(&buffer);
            // all map entities will have the object id of the database region, this value is the same for all map entities in a region
            map_entity.object_id = region_object_id;
            region_tiles.insert(map_entity.id.clone(), map_entity);

            start = end;
            end = end + 69;

            if end > size
            {
                break;
            }
            // counting mapentities
            count += 1;

        }
        regions_data.push((region_id.clone(), region_tiles));
    }

    println!("finished loading data, starting services tiles: {}", count);
    regions_data
    // GameMap::new(regions_data)
}

async fn get_compressed_tiles_data_from_file(world_id : &str, region_id : String) -> Vec<u8> {
    let file_name = format!("map_initial_data/{}_{}_props.bytes",world_id, region_id);
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));
    println!("reading file {}", file_name);

    let tiles = tokio::fs::read(file_name).await.unwrap();
    let size = tiles.len();

    let mut buffer = [0u8;69];
    let mut start = 0;
    let mut end = 69;

    loop {
        buffer.copy_from_slice(&tiles[start..end]);
        encoder.write_all(&buffer).unwrap();

        start = end;
        end = end + 69;
        if end > size
        {
            break;
        }
    }

    let compressed_bytes = encoder.reset(Vec::new()).unwrap();
    compressed_bytes
}

async fn load_files_into_regions_hashset(world_id : &str) -> HashMap<TetrahedronId, Vec<u8>> {

    let encoded_areas : [char; 20] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't'];

    let initial_tiles : Vec<TetrahedronId> = encoded_areas.map(|l| {
        let first = l.to_string();
        TetrahedronId::from_string(&first)
    }).into_iter().collect();


    let mut regions = Vec::<TetrahedronId>::new();
    for initial in initial_tiles
    {
        get_regions(initial, 2, &mut regions);
    }

    let mut regions_data = HashMap::<TetrahedronId, Vec<u8>>::new();
    for region in regions
    {
        let data = get_compressed_tiles_data_from_file(world_id, region.to_string()).await;
        regions_data.insert(region, data);
    }

    regions_data
}

