use std::collections::HashMap;
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
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time::error::Elapsed;

// #[tokio::main(worker_threads = 1)]
#[tokio::main()]
async fn main() {

    let (_tx, mut rx) = tokio::sync::watch::channel("hello");
    //console_subscriber::init();
    // tiles are modified by many systems, but since we only have one core... our mutex doesn't work too much
    // let all_tiles = HashMap::<TetrahedronId,MapEntity>::new();
    let working_tiles = load_files(true).await;
    let storage_tiles = load_files(false).await;

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

async fn get_tiles_from_file(region_id : String, all_tiles : &mut HashMap<TetrahedronId, MapEntity>, save_compressed : bool){
    let file_name = format!("map_initial_data/world_002_{}_props.bytes", region_id);
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));
    println!("reading file {}", file_name);

    let tiles = tokio::fs::read(file_name).await.unwrap();
    let size = tiles.len();

    let mut buffer = [0u8;69];
    let mut start = 0;
    let mut end = 69;
    println!("initialy for region {} {}",region_id, all_tiles.len());

    let mut count = 0;
    let mut limit = 13517;
    let mut test_tiles_original : HashMap<TetrahedronId, MapEntity> = HashMap::new();

    loop {
        buffer.copy_from_slice(&tiles[start..end]);
        if save_compressed {
            encoder.write_all(&buffer).unwrap();
        }
        let map_entity = MapEntity::from_bytes(&buffer);
        // if test_tiles_original.contains_key(&map_entity.id)
        // {
        //     println!("we have a dup somehow in original {} ", map_entity.id);
        // }
        // else
        // {
        //     test_tiles_original.insert(map_entity.id.clone(), map_entity.clone());
        // }

        all_tiles.insert(map_entity.id.clone(), map_entity);
        // println!("{:?}", map_entity);
        start = end;
        end = end + 69;
        count += 1;

        if end > size
        {
            break;
        }
        limit -= 1;

        if limit <= 0
        {
            // break;
        }

    }
    println!("encoded data count {}" , count);
    // println!("end {} for region {}" , all_tiles.len(), region_id);


    if save_compressed {
        let compressed_bytes = encoder.reset(Vec::new()).unwrap();
        let file_name = format!("map_working_data/world_002_{}_props.bytes", region_id.to_string());
        let mut file = File::create(file_name).await.unwrap();
        file.write_all(&compressed_bytes).await.unwrap();


        // let file_name = format!("map_working_data/world_002_{}_props.bytes", region_id.to_string());
        // let tiles = tokio::fs::read(file_name).await.unwrap();
        // let compressed_size = tiles.len();

        // let mut decoder = ZlibDecoder::new(tiles.as_slice());

        // let decoded_data_result :  Result<Vec<u8>, _> = std::io::Read::bytes(decoder).collect();
        // let decoded_data = decoded_data_result.unwrap();

        // let decoded_data_size = decoded_data.len();
        // println!("After saving region size of compressed {} vs original {} vs decoded {}",compressed_size, size, decoded_data_size);


        // let mut start = 0;
        // let mut end = 69;
        // let mut count = 0;
        // let mut test_tiles : HashMap<TetrahedronId, MapEntity> = HashMap::new();
        // loop {
        //     buffer.copy_from_slice(&decoded_data[start..end]);
        //     let map_entity = MapEntity::from_bytes(&buffer);
        //     if test_tiles.contains_key(&map_entity.id)
        //     {
        //         println!("we have a dup somehow {} ", map_entity.id);
        //     }
        //     else
        //     {
        //         test_tiles.insert(map_entity.id.clone(), map_entity);
        //     }
        //     // println!("got a tile {}", map_entity.id);
        //     start = end;
        //     end = end + 69;
        //     count += 1;

        //     if end > decoded_data_size
        //     {
        //         break;
        //     }
        // }
        // println!("decoded data count {}" , count);

    }

}

async fn load_files(save_compressed : bool) -> GameMap {

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
        get_tiles_from_file(region.to_string(), &mut region_tiles, save_compressed).await;
        regions_data.push((region, region_tiles));
        // break;
    }

    println!("finished loading data, starting services tiles: {}", len);
    GameMap::new(regions_data)
}

