use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;

use flate2::read::ZlibDecoder;
use game_server::long_term_storage_service;
use game_server::map::GameMap;
use game_server::map::map_entity::MapCommand;
use game_server::map::map_entity::MapEntity;
use game_server::map::tetrahedron_id::TetrahedronId;
use game_server::real_time_service;
use game_server::web_service;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use mongodb::Client;
use mongodb::options::ClientOptions;
use mongodb::options::ResolverConfig;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time::error::Elapsed;

// #[tokio::main(worker_threads = 1)]
#[tokio::main()]
async fn main() {

    let (_tx, mut rx) = tokio::sync::watch::channel("hello");

    let client_uri = "mongodb://localhost:27017/test?retryWrites=true&w=majority";
    let options = ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare()).await.unwrap();
    let db_client = Client::with_options(options).unwrap();

    // tiles are modified by many systems, but since we only have one core... our mutex doesn't work too much
    let world_name = "world_002";

    let mut working_tiles: Option<GameMap> = None; // load_files_into_game_map(world_name).await;
    let mut storage_tiles: Option<GameMap> = None; // load_files_into_game_map(world_name).await;

    let world_state = long_term_storage_service::check_world_state(world_name, db_client.clone()).await;

    if let Some(world) = world_state {
        println!("Load the world from db init at {}", world.start_time);
        let regions_data = long_term_storage_service::get_regions_from_db(world_name, db_client.clone()).await;
        println!("reading regions into game maps");
        let game_map_1 = load_regions_data_into_game_map(&regions_data);
        let game_map_2 = load_regions_data_into_game_map(&regions_data);
        working_tiles = Some(game_map_1);
        storage_tiles = Some(game_map_2);
    }
    else{
        println!("Creating world from scratch, because it was not found in the database");
        // any errors will just crash the app.

        let world_id = long_term_storage_service::init_world_state(world_name, db_client.clone()).await;
        if let Some(id) = world_id{
            println!("Creating world with id {}", id);
            let regions_data = load_files_into_regions_hashset(world_name).await;
            let game_map_1 = load_regions_data_into_game_map(&regions_data);
            let game_map_2 = load_regions_data_into_game_map(&regions_data);
            working_tiles = Some(game_map_1);
            storage_tiles = Some(game_map_2);
            long_term_storage_service::preload_db(world_name, world_id, regions_data, db_client.clone()).await;
        }
        else {
            println!("Error creating world in db");
            return;
        }
    }

    // tiles mirrow image
    let (tile_update_tx, tile_update_rx ) = tokio::sync::mpsc::channel::<MapEntity>(100);
    let (map_command_tx, real_time_service_rx ) = tokio::sync::mpsc::channel::<MapCommand>(20);
    let web_service_map_commands_tx = map_command_tx.clone();
    let client_map_commands_tx = map_command_tx.clone();

    match (working_tiles, storage_tiles) {
        (Some(working_tiles), Some(storage_tiles)) =>
        {
            let working_tiles_reference= Arc::new(working_tiles);
            long_term_storage_service::start_server(tile_update_rx, storage_tiles, db_client.clone());
            web_service::start_server(working_tiles_reference.clone(), web_service_map_commands_tx, db_client.clone());
            real_time_service::start_server(
                working_tiles_reference.clone(), 
                client_map_commands_tx, 
                real_time_service_rx,
                tile_update_tx,
            );
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

async fn get_tiles_from_file(world_id : &str, region_id : String, all_tiles : &mut HashMap<TetrahedronId, MapEntity>){
    let file_name = format!("map_initial_data/{}_{}_props.bytes",world_id, region_id);
    println!("reading file {}", file_name);

    let tiles = tokio::fs::read(file_name).await.unwrap();
    let size = tiles.len();

    let mut buffer = [0u8;69];
    let mut start = 0;
    let mut end = 69;
    println!("initialy for region {} {}",region_id, all_tiles.len());

    let test_tiles_original : HashMap<TetrahedronId, MapEntity> = HashMap::new();

    loop {
        buffer.copy_from_slice(&tiles[start..end]);
        let map_entity = MapEntity::from_bytes(&buffer);
        all_tiles.insert(map_entity.id.clone(), map_entity);

        start = end;
        end = end + 69;

        if end > size
        {
            break;
        }
    }
}

async fn load_files_into_game_map(world_id : &str) -> GameMap {

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
    let len = regions.len();

    let mut regions_data = Vec::<(TetrahedronId, HashMap<TetrahedronId, MapEntity>)>::new();
    for region in regions
    {
        let mut region_tiles = HashMap::<TetrahedronId,MapEntity>::new();
        // println!("get data for region {}", region.to_string());
        // world_002
        get_tiles_from_file(world_id, region.to_string(), &mut region_tiles).await;
        regions_data.push((region, region_tiles));
    }

    println!("finished loading data, starting services tiles: {}", len);
    GameMap::new(regions_data)
}

fn load_regions_data_into_game_map(regions_stored_data : &HashMap<TetrahedronId, Vec<u8>>) -> GameMap {

    let mut regions_data = Vec::<(TetrahedronId, HashMap<TetrahedronId, MapEntity>)>::new();

    let mut count = 0;
    let mut region_count = 0;
    let mut region_total = regions_stored_data.len();

    for region in regions_stored_data{
        println!("decoding region {} progress {}/{}", region.0, region_count, region_total);
        region_count += 1;
        let region_id = region.0;
        let data : &[u8] = region.1;
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
            let map_entity = MapEntity::from_bytes(&buffer);
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
    GameMap::new(regions_data)
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
    let len = regions.len();

    let mut regions_data = HashMap::<TetrahedronId, Vec<u8>>::new();
    for region in regions
    {
        let data = get_compressed_tiles_data_from_file(world_id, region.to_string()).await;
        regions_data.insert(region, data);
    }

    regions_data
}

