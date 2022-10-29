use std::collections::HashMap;
use std::sync::Arc;

use game_server::long_term_storage_service;
use game_server::map::GameMap;
use game_server::map::map_entity::MapCommand;
use game_server::map::map_entity::MapEntity;
use game_server::map::tetrahedron_id::TetrahedronId;
use game_server::real_time_service;
use game_server::web_service;

// #[tokio::main(worker_threads = 1)]
#[tokio::main()]
async fn main() {

    let (_tx, mut rx) = tokio::sync::watch::channel("hello");
    //console_subscriber::init();
    // tiles are modified by many systems, but since we only have one core... our mutex doesn't work too much
    // let all_tiles = HashMap::<TetrahedronId,MapEntity>::new();
    let working_tiles = load_files().await;
    let storage_tiles = load_files().await;

    let working_tiles_reference= Arc::new(working_tiles);


    // tiles mirrow image
    let (tile_update_tx, tile_update_rx ) = tokio::sync::mpsc::channel::<MapEntity>(100);

    let (map_command_tx, real_time_service_rx ) = tokio::sync::mpsc::channel::<MapCommand>(20);
    let web_service_map_commands_tx = map_command_tx.clone();
    let client_map_commands_tx = map_command_tx.clone();


    long_term_storage_service::start_server(tile_update_rx, storage_tiles);

    real_time_service::start_server(
        working_tiles_reference.clone(), 
        client_map_commands_tx, 
        real_time_service_rx,
        tile_update_tx,
    );

    web_service::start_server(working_tiles_reference.clone(), web_service_map_commands_tx);

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

async fn get_tiles_from_file(code : String, all_tiles : &mut HashMap<TetrahedronId, MapEntity>){
    let file_name = format!("map_initial_data/world001_{}_props.bytes", code);
    println!("reading file {}", file_name);

    let tiles = tokio::fs::read(file_name).await.unwrap();
    let size = tiles.len();

    let mut buffer = [0u8;66];
    let mut start = 0;
    let mut end = 66;

    loop {
        buffer.copy_from_slice(&tiles[start..end]);
        let map_entity = MapEntity::from_bytes(&buffer);
        all_tiles.insert(map_entity.id.clone(), map_entity);
        // println!("{:?}", map_entity);
        start = end;
        end = end + 66;

        if end > size
        {
            break;
        }
    }
}

async fn load_files() -> GameMap {

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
        get_tiles_from_file(region.to_string(), &mut region_tiles).await;
        regions_data.push((region, region_tiles));
    }

    println!("finished loading data, starting services tiles: {}", len);
    GameMap::new(regions_data)
}

