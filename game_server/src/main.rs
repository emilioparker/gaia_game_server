use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::AtomicU32;
use game_server::get_regions_by_code;
use game_server::get_regions_by_id;
use strum;

use cli_log::init_cli_log;
use flate2::read::ZlibDecoder;
use game_server::app;
use game_server::definitions::buffs_data::BuffData;
use game_server::definitions::card::Card;
use game_server::definitions::character_progression::CharacterProgression;
use game_server::definitions::definition_versions::DefinitionVersion;
use game_server::definitions::definitions_container::Definitions;
use game_server::definitions::definitions_container::DefinitionsData;
use game_server::definitions::items::Item;
use game_server::definitions::main_paths::MapPath;
use game_server::definitions::mob_progression::MobProgression;
use game_server::definitions::mobs_data::MobData;
use game_server::definitions::props_data::PropData;
use game_server::definitions::tower_difficulty::TowerDifficulty;
use game_server::definitions::weapons::Weapon;
use game_server::definitions::Definition;
use game_server::AppData;
use game_server::ServerChannels;
use game_server::ServerState;
use game_server::chat_service;
use game_server::gameplay_service;
use game_server::long_term_storage_service;
use game_server::long_term_storage_service::db_region::StoredRegion;
use game_server::map::GameMap;
use game_server::map::map_entity::MAP_ENTITY_SIZE;
use game_server::map::map_entity::MapEntity;
use game_server::map::tetrahedron_id::TetrahedronId;
use game_server::clients_service;
use game_server::web_service;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use hyper_static::serve;
use mongodb::options::ServerApiVersion;
use mongodb::Client;
use mongodb::options::ClientOptions;
use mongodb::options::ResolverConfig;

use strum::IntoEnumIterator;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::Sender;

use std::panic::{set_hook, take_hook};

fn main() 
{
    //GAIA_LOG=debug cargo run --release
    init_cli_log!("gaia");
    // build runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("gaia-main_thread")
        .thread_stack_size(3 * 1024 * 1024)
        .enable_all()
        .build()
        .unwrap();


    let (tx, rx) = oneshot::channel();
    cli_log::info!("running server");
    runtime.spawn(run_server(tx)); 
    // runtime.block_on(run_server(tx)); 
    cli_log::info!("running tui");
    init_panic_hook();
    runtime.block_on(run_tui(rx)); 
    cli_log::info!("--end--");
}

async fn run_tui(rx: Receiver<AppData>)
{
    if let Ok(game_data) = rx.await 
    {
        let terminal = ratatui::init();
        let result = app::App::new(game_data).run(terminal);
        ratatui::restore();
    }
    // let terminal = ratatui::init();
    // let result = app::App::new().run(terminal);
    // ratatui::restore();
}

pub fn init_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| 
    {
        // intentionally ignore errors here since we're already in a panic
        let _ = restore_tui();
        original_hook(panic_info);
    }));
}

pub fn restore_tui() -> std::io::Result<()> 
{
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}

// #[tokio::main(worker_threads = 1)]
// #[tokio::main()]
async fn run_server(tx: Sender<AppData>) 
{
    let mut main_loop = tokio::time::interval(std::time::Duration::from_millis(50000));

    let definitions = load_definitions().await;

    let mut channels_status = HashMap::new();
    for channel in ServerChannels::iter() 
    {
        channels_status.insert(channel, AtomicU16::new(0));
    }

    let server_state = Arc::new(ServerState
    {
        channels: channels_status,
        online_players:AtomicU32::new(0),
        total_players:AtomicU32::new(0),
        received_packets: AtomicU64::new(0),
        received_bytes: AtomicU64::new(0),
        sent_udp_packets: AtomicU64::new(0),
        sent_game_packets: AtomicU64::new(0),
        sent_bytes: AtomicU64::new(0),

        //char
        pending_character_entities_to_save: AtomicU32::new(0),
        saved_character_entities: AtomicU32::new(0),
        last_character_entities_save_timestamp: AtomicU64::new(0),

        //map
        pending_regions_to_save: AtomicU32::new(0),
        saved_regions: AtomicU32::new(0),
        last_regions_save_timestamp: AtomicU64::new(0),

        //towers
        pending_tower_entities_to_save: AtomicU32::new(0),
        saved_tower_entities: AtomicU32::new(0),
        last_tower_entities_save_timestamp: AtomicU64::new(0),
    });
    // let (_tx, mut rx) = tokio::sync::watch::channel("hello");

    let client_uri = "mongodb://localhost:27017/test?retryWrites=true&w=majority";
    let options = ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare()).await.unwrap();
    let db_client = Client::with_options(options).unwrap();

    let world_name = "world_082";

    let working_game_map: Option<GameMap>; // load_files_into_game_map(world_name).await;
    let storage_game_map: Option<GameMap>; // load_files_into_game_map(world_name).await;

    let world_state = long_term_storage_service::world_service::check_world_state(world_name, db_client.clone()).await;


    let mut presentation_data_cache : Vec<u8> = Vec::new();
    //shared by the realtime service and the webservice

    if let Some(world) = world_state 
    {
        cli_log::info!("Load the world from db init at {}", world.start_time);
        let working_players = long_term_storage_service::heroes_service::get_heroes_from_db_by_world(world.id, db_client.clone()).await;
        //used and updated by the long storage system
        let storage_players = working_players.clone();

        let regions_db_data = long_term_storage_service::world_service::get_regions_from_db(world.id, db_client.clone()).await;
        cli_log::info!("reading regions into game maps");
        let regions_data = load_regions_data_into_game_map(&regions_db_data);


        for (_id, player) in &working_players
        {
            let name_with_padding = format!("{: <5}", player.hero_name);
            let name_data : Vec<u32> = name_with_padding.chars().into_iter().map(|c| c as u32).collect();
            let mut name_array = [0u32; 5];
            name_array.clone_from_slice(&name_data.as_slice()[0..5]);

            let player_presentation = game_server::hero::hero_presentation::HeroPresentation 
            {
                player_id: player.hero_id,
                character_name: name_array,
            };
            cli_log::info!("Adding player data {}", player.hero_name);

            presentation_data_cache.extend(player_presentation.to_bytes());
        }
        // let (towers, _) = load_files_into_regions_hashset(world_name).await;
        // long_term_storage_service::towers_service::preload_db(world_name, world.id, towers, db_client.clone()).await;
        let world_towers = long_term_storage_service::towers_service::get_towers_from_db_by_world(world.id, db_client.clone()).await;

        working_game_map = Some(GameMap::new(world.id, world.world_name.clone(), definitions.0.clone(), regions_data.clone(), working_players, world_towers.clone()));
        storage_game_map = Some(GameMap::new(world.id, world.world_name, definitions.0, regions_data, storage_players, world_towers));

    }
    else
    {
        cli_log::info!("Creating world from scratch, because it was not found in the database");
        // any errors will just crash the app.

        let world_id = long_term_storage_service::world_service::init_world_state(world_name, db_client.clone()).await;
        if let Some(id) = world_id
        {
            cli_log::info!("Creating world with id {}", id);
            let working_players = long_term_storage_service::heroes_service::get_heroes_from_db_by_world(world_id, db_client.clone()).await;
            //used and updated by the long storage system
            let storage_players = working_players.clone();

            let (towers,regions_data) = load_files_into_regions_hashset(world_name).await;

            long_term_storage_service::world_service::preload_db(world_name, world_id, regions_data, db_client.clone()).await;
            long_term_storage_service::towers_service::preload_db(world_name, world_id, towers, db_client.clone()).await;

            // reading what we just created because we need the object ids!
            let regions_data_from_db = long_term_storage_service::world_service::get_regions_from_db(world_id, db_client.clone()).await;

            let regions_data = load_regions_data_into_game_map(&regions_data_from_db);

            let world_towers = long_term_storage_service::towers_service::get_towers_from_db_by_world(world_id, db_client.clone()).await;

            working_game_map = Some(GameMap::new(world_id, world_name.to_string(),definitions.0.clone(), regions_data.clone(), working_players, world_towers.clone()));
            storage_game_map = Some(GameMap::new(world_id, world_name.to_string(),definitions.0, regions_data, storage_players, world_towers));
        }
        else {
            cli_log::info!("Error creating world in db");
            return;
        }
    }

    match (working_game_map, storage_game_map) {
        (Some(working_game_map), Some(storage_game_map)) =>
        {
            let working_game_map_reference= Arc::new(working_game_map);
            let storage_game_map_reference= Arc::new(storage_game_map);
            let tui_game_map_reference= storage_game_map_reference.clone();

            let (
                rx_mc_client_gameplay,
                rx_moc_client_gameplay, 
                rx_hc_client_gameplay, 
                rx_tc_client_gameplay ,
                rx_cc_client_gameplay ,
                tx_packets_gameplay_chat_clients,
            ) =  clients_service::start_server(
                working_game_map_reference.clone(), 
                server_state.clone());
                

            let (rx_me_gameplay_longterm,
                rx_me_gameplay_webservice,
                rx_moe_gameplay_webservice,
                rx_he_gameplay_longterm,
                rx_te_gameplay_longterm,
                rx_te_gameplay_webservice,
                _tx_mc_webservice_gameplay,
            ) = gameplay_service::start_service(
                rx_hc_client_gameplay,
                rx_mc_client_gameplay,
                rx_moc_client_gameplay,
                rx_tc_client_gameplay,
                working_game_map_reference.clone(), 
                server_state.clone(),
                tx_packets_gameplay_chat_clients.clone());

            let rx_ce_gameplay_webservice = chat_service::start_service(
                rx_cc_client_gameplay,
                working_game_map_reference.clone(), 
                server_state.clone(),
                tx_packets_gameplay_chat_clients);

            // realtime service sends the mapentity after updating the working copy, so it can be stored eventually
            let rx_me_saved_longterm_web= long_term_storage_service::world_service::start_server(
                rx_me_gameplay_longterm,
                storage_game_map_reference.clone(), 
                server_state.clone(),
                db_client.clone()
            );

            long_term_storage_service::heroes_service::start_server(
                rx_he_gameplay_longterm,
                storage_game_map_reference.clone(), 
                server_state.clone(),
                db_client.clone()
            );

            // realtime service sends the mapentity after updating the working copy, so it can be stored eventually
            let rx_te_saved_longterm_web = long_term_storage_service::towers_service::start_server(
                rx_te_gameplay_longterm,
                storage_game_map_reference.clone(), 
                server_state.clone(),
                db_client.clone()
            );
            
            web_service::start_server
            (
                presentation_data_cache,
                working_game_map_reference, 
                storage_game_map_reference, 
                server_state.clone(),
                db_client.clone(),
                definitions.1,
                rx_me_gameplay_webservice,
                rx_moe_gameplay_webservice,
                rx_te_gameplay_webservice,
                rx_ce_gameplay_webservice,
                rx_me_saved_longterm_web,
                rx_te_saved_longterm_web,
            );
        // ---------------------------------------------------

            if let Ok(_) = tx.send(AppData{
                game_data: tui_game_map_reference,
                game_status: server_state.clone(),
            })
            {
                cli_log::info!("Data sent to tui");
            }
            else {
                cli_log::error!("Error sending data to tui");
            };
        },
        _ => {
            cli_log::info!("big and horrible error with the working and storage tiles");
        }
    }

    cli_log::info!("Game server started correctly");

    loop 
    {
        main_loop.tick().await;
    }

}


async fn load_definition_by_name<T>(file_name : String) -> (Vec<T>, Vec<u8>)
where T: serde::de::DeserializeOwned + Definition
{
    let file_name = format!("definitions/{file_name}");
    let mut data = Vec::<T>::new();
    cli_log::info!("reading definition file {}", file_name);
    let definition_versions_data = tokio::fs::read(file_name).await.unwrap();
    let mut rdr = csv::Reader::from_reader(definition_versions_data.as_slice());
    for result in rdr.deserialize() 
    {
        let record: T = result.unwrap();
        data.push(record);
    }
    (data, definition_versions_data)
}

async fn load_definitions() -> (Definitions, DefinitionsData)
{
    let mut definition_versions = HashMap::new();

    let file_name = format!("definition_versions.csv");
    let definition_versions_result = load_definition_by_name::<DefinitionVersion>(file_name).await;

    for entry in definition_versions_result.0
    {
        definition_versions.insert(entry.key.clone(), entry);
    }

    let file_name = format!("character_progression.csv");
    let character_result = load_definition_by_name::<CharacterProgression>(file_name).await;

    let file_name = format!("mob_progression.csv");
    let mob_progression_result = load_definition_by_name::<MobProgression>(file_name).await;

    let file_name = format!("props.csv");
    let props_result = load_definition_by_name::<PropData>(file_name).await;

    let file_name = format!("main_paths.csv");
    let paths_result = load_definition_by_name::<MapPath>(file_name).await;

    let file_name = format!("towers_difficulty.csv");
    let towers_difficulty_result = load_definition_by_name::<TowerDifficulty>(file_name).await;

    let file_name = format!("items.csv");
    let items_result = load_definition_by_name::<Item>(file_name).await;

    let file_name = format!("cards.csv");
    let cards_result = load_definition_by_name::<Card>(file_name).await;

    let file_name = format!("mobs.csv");
    let mobs_result = load_definition_by_name::<MobData>(file_name).await;

    let file_name = format!("buffs.csv");
    let buffs_result = load_definition_by_name::<BuffData>(file_name).await;

    let file_name = format!("weapons.csv");
    let weapons_result = load_definition_by_name::<Weapon>(file_name).await;

    let mut buffs_hash = HashMap::new();

    for entry in &buffs_result.0
    {
        buffs_hash.insert(entry.id.clone(), entry.clone());
    }

    let definitions = Definitions 
    {
        regions_by_id: get_regions_by_id(),
        regions_by_code: get_regions_by_code(),
        character_progression : character_result.0,
        props : props_result.0,
        mob_progression : mob_progression_result.0,
        main_paths: paths_result.0,
        towers_difficulty: towers_difficulty_result.0,
        items: items_result.0,
        cards :cards_result.0,
        mobs: mobs_result.0,
        buffs_by_code: buffs_result.0,
        buffs : buffs_hash,
        weapons : weapons_result.0,
    };

    let definitions_data = DefinitionsData
    {
        definition_versions,
        character_progression_data : character_result.1,
        mob_progression_data : mob_progression_result.1,
        definition_versions_data : definition_versions_result.1,
        props_data : props_result.1,
        main_paths_data : paths_result.1,
        towers_difficulty_data: towers_difficulty_result.1,
        items_data :items_result.1,
        cards_data: cards_result.1,
        mobs_data: mobs_result.1,
        buffs_data: buffs_result.1,
        weapons_data: weapons_result.1,
    };

    (definitions, definitions_data)
}


fn load_regions_data_into_game_map(
    regions_stored_data : &HashMap<TetrahedronId, StoredRegion>
) 
-> Vec<(TetrahedronId, HashMap<TetrahedronId, MapEntity>)> 
{
    let mut regions_data = Vec::<(TetrahedronId, HashMap<TetrahedronId, MapEntity>)>::new();

    let mut count = 0;
    let mut region_count = 0;
    let region_total = regions_stored_data.len();

    for region in regions_stored_data.iter()
    {
        region_count += 1;
        // cli_log::info!("decoding region progress {region_count}/{region_total} tiles {count}");

        let region_object_id = region.1.id.clone();
        let binary_data: Vec<u8> = match region.1.compressed_data.clone() 
        {
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

        // cli_log::info!("initialy for region {} {}",region_id, all_tiles.len());

        let mut region_tiles : HashMap<TetrahedronId, MapEntity> = HashMap::new();

        loop {
            buffer.copy_from_slice(&tiles[start..end]);
            let mut map_entity = MapEntity::from_bytes(&buffer);
            // all map entities will have the object id of the database region, this value is the same for all map entities in a region
            map_entity.object_id = region_object_id;
            
            if map_entity.id.to_string() == "j202020303" {
                cli_log::info!("Found saved entity  {:?} " , map_entity);
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

    cli_log::info!("finished loading data, starting services. regions: {} with {} tiles",region_total, count);
    regions_data
    // GameMap::new(regions_data)
}

async fn get_compressed_tiles_data_from_file(world_id : &str, region_id : String) -> (Vec<TetrahedronId>, Vec<u8>)
{
    let file_name = format!("../../map_initial_data/{}_{}_props.bytes",world_id, region_id);
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(9));
    cli_log::info!("reading file {}", file_name);

    let tiles = tokio::fs::read(file_name).await.unwrap();
    let size = tiles.len();

    let mut buffer = [0u8;MAP_ENTITY_SIZE];
    let mut start = 0;
    let mut end = MapEntity::get_size();

    let mut towers_in_region = Vec::new();

    loop 
    {
        buffer.copy_from_slice(&tiles[start..end]);
        encoder.write_all(&buffer).unwrap();
        let tile = MapEntity::from_bytes(&buffer);
        if tile.prop == 35
        {
            towers_in_region.push(tile.id.clone());
        }

        start = end;
        end = end + MapEntity::get_size();
        if end > size
        {
            break;
        }
    }

    let compressed_bytes = encoder.reset(Vec::new()).unwrap();
    (towers_in_region, compressed_bytes)
}

async fn load_files_into_regions_hashset(world_id : &str) -> (Vec<TetrahedronId>, HashMap<TetrahedronId, Vec<u8>>) 
{
    let mut towers = Vec::new();
    let regions = game_server::map::get_region_ids(2);
    let mut regions_data = HashMap::<TetrahedronId, Vec<u8>>::new();
    for region in regions
    {
        let (mut towers_in_region, data) = get_compressed_tiles_data_from_file(world_id, region.to_string()).await;
        regions_data.insert(region, data);
        towers.append(&mut towers_in_region);
    }
    (towers, regions_data)
}


#[cfg(test)]
mod tests {
    use std::{io::Write, collections::HashMap};
    
    use game_server::{long_term_storage_service::db_region::StoredRegion, map::{tetrahedron_id::TetrahedronId, map_entity::MapEntity}};
    use mongodb::{Client, options::{ClientOptions, ResolverConfig}};
    use mongodb::bson::doc;
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
        // cli_log::info!("reading regions into game maps");
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

        cli_log::info!("delete result {delete_result:?}");

        // let mut locked_tiles = region.lock().await;

        let old = region.get(&tile_id);
        match old {
            Some(previous_record) => {
                cli_log::info!("got a {:?}", previous_record);
                let new_tile = MapEntity{
                    health: 50,
                    ..previous_record.clone()
                };
                region.insert(tile_id.clone(), new_tile);
            }
            _ => {
                cli_log::info!("not found");
                assert!(false);
                // locked_tiles.insert(message.id.clone(), message);
            }
        }
        // checking if update worked.

        let old = region.get(&tile_id);
        match old {
            Some(previous_record) => {
                cli_log::info!("got a {:?}", previous_record);
                assert!(previous_record.health == 50);
            }
            _ => {
                cli_log::info!("not found");
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

        cli_log::info!("update_result {insert_result:?}");

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
        assert!(tile.health == 50);

        let old = region.get(&tile_id);
        match old {
            Some(previous_record) => {
                cli_log::info!("got a {:?}", previous_record);
                let new_tile = MapEntity{
                    health: 20,
                    ..previous_record.clone()
                };
                region.insert(tile_id.clone(), new_tile);
            }
            _ => {
                cli_log::info!("not found");
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
        
        cli_log::info!("update_result {update_result:?}");


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
