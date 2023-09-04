use std::collections::HashMap;
use std::sync::Arc;

use crate::ServerState;
use crate::character::character_attack::CharacterAttack;
use crate::character::character_command::CharacterCommand;
use crate::map::GameMap;
use crate::map::map_entity::MapCommand;
use crate::character::character_entity::CharacterEntity;
use crate::character::character_reward::CharacterReward;
use crate::map::map_entity::MapEntity;
use crate::real_time_service::client_handler::StateUpdate;
use crate::tower::{TowerCommand, TowerCommandInfo};
use crate::tower::tower_entity::TowerEntity;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

use self::utils::{get_tile_commands_to_execute, get_player_commands_to_execute, report_tower_process_capacity, get_tower_commands_to_execute};
pub mod data_packer;
pub mod utils;
pub mod player_commands_processor;
pub mod tile_commands_processor;
pub mod tower_commands_processor;

pub enum DataType
{
    NoData = 25,
    PlayerState = 26,
    TileState = 27,
    PlayerPresentation = 28,
    PlayerAttack = 29,
    PlayerReward = 30,
    TileAttack = 31,
    TowerState = 32,
}

pub fn start_service(
    mut rx_pc_client_game : tokio::sync::mpsc::Receiver<CharacterCommand>,
    mut rx_mc_client_game : tokio::sync::mpsc::Receiver<MapCommand>,
    mut rx_tc_client_game : tokio::sync::mpsc::Receiver<TowerCommand>,
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    tx_bytes_game_socket: tokio::sync::mpsc::Sender<Vec<(u64, Vec<u8>)>>
) 
-> (Receiver<MapEntity>, Receiver<MapEntity>, Receiver<CharacterEntity>, Receiver<TowerEntity>, Receiver<TowerEntity>, Sender<MapCommand>) 
{

    let (tx_mc_webservice_gameplay, mut _rx_mc_webservice_gameplay ) = tokio::sync::mpsc::channel::<MapCommand>(200);
    let (tx_me_gameplay_longterm, rx_me_gameplay_longterm ) = tokio::sync::mpsc::channel::<MapEntity>(1000);
    let (tx_me_gameplay_webservice, rx_me_gameplay_webservice) = tokio::sync::mpsc::channel::<MapEntity>(1000);
    let (tx_pe_gameplay_longterm, rx_pe_gameplay_longterm ) = tokio::sync::mpsc::channel::<CharacterEntity>(1000);
    let (tx_te_gameplay_longterm, rx_te_gameplay_longterm ) = tokio::sync::mpsc::channel::<TowerEntity>(100);
    let (tx_te_gameplay_webservice, rx_te_gameplay_webservice) = tokio::sync::mpsc::channel::<TowerEntity>(100);

    //players
    //player commands -------------------------------------
    let player_commands = Vec::<CharacterCommand>::new();
    let player_commands_mutex = Arc::new(Mutex::new(player_commands));
    let player_commands_processor_lock = player_commands_mutex.clone();
    let player_commands_agregator_lock = player_commands_mutex.clone();

    //tile commands -------------------------------------
    let tile_commands = Vec::<MapCommand>::new();
    let tile_commands_mutex = Arc::new(Mutex::new(tile_commands));
    let tile_commands_processor_lock = tile_commands_mutex.clone();

    let tile_commands_agregator_from_client_lock = tile_commands_mutex.clone();
    // let tile_commands_agregator_from_webservice_lock = tile_commands_mutex.clone();

    //tower commands -------------------------------------
    let tower_commands = Vec::<TowerCommand>::new();
    let tower_commands_mutex = Arc::new(Mutex::new(tower_commands));
    let tower_commands_processor_lock = tower_commands_mutex.clone();

    let tower_commands_agregator_from_client_lock = tower_commands_mutex.clone();

    //delayed commands for attacks so they struck a bit later.
    let delayed_tile_commands = Vec::<(u64, MapCommand)>::new();
    let delayed_tile_commands_mutex = Arc::new(Mutex::new(delayed_tile_commands));
    let delayed_tile_commands_lock = delayed_tile_commands_mutex.clone();

    //delayed commands for attacks so they struck a bit later.
    let delayed_player_commands = Vec::<(u64, u16)>::new();
    let delayed_player_commands_mutex = Arc::new(Mutex::new(delayed_player_commands));
    let delayed_player_commands_lock = delayed_player_commands_mutex.clone();

    //delayed commands for attacks so they struck a bit later.
    let delayed_tower_commands = Vec::<(u64, TowerCommand)>::new();
    let delayed_tower_commands_mutex = Arc::new(Mutex::new(delayed_tower_commands));
    let delayed_tower_commands_lock = delayed_tower_commands_mutex.clone();

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));

    //task that will handle receiving state changes from clients and updating the global statestate.
    tokio::spawn(async move {

        loop {
            let message = rx_pc_client_game.recv().await.unwrap();

            // println!("got a player change data {}", message.player_id);
            // let mut current_time = 0;
            // let result = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
            // if let Ok(elapsed) = result {
            //     current_time = elapsed.as_secs();
            // }

            let mut data = player_commands_agregator_lock.lock().await;
            
            data.push(message);
        }
    });

    // task that gathers world changes comming from a client into a list.
    tokio::spawn(async move {
        // let mut sequence_number:u64 = 101;
        loop {
            let message = rx_mc_client_game.recv().await.unwrap();
            // println!("got a tile change data {}", message.id);
            let mut data = tile_commands_agregator_from_client_lock.lock().await;
            data.push(message);
        }
    });

    tokio::spawn(async move {

        // let mut sequence_number:u64 = 101;
        loop {
            let message = rx_tc_client_game.recv().await.unwrap();
            println!("got a tower change data {}", message.id);
            let mut data = tower_commands_agregator_from_client_lock.lock().await;
            data.push(message);
        }
    });

    // task that gathers world changes comming from web service into a list.
    // tokio::spawn(async move {
    //     // let mut sequence_number:u64 = 101;
    //     loop {
    //         let message = rx_mc_webservice_gameplay.recv().await.unwrap();
    //         // println!("got a tile change data {}", message.id);
    //         let mut data = tile_commands_agregator_from_webservice_lock.lock().await;
    //         data.push(message);
    //     }
    // });

    // task that will perdiodically send dta to all clients
    tokio::spawn(async move {
        let mut packet_number = 1u64;
        loop {
            // assuming 30 fps.
            // tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            interval.tick().await;

            // let result = std::time::SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
            // let current_time = result.ok().map(|d| d.as_millis() as u64);

            // let time = &map.time;
            // if let Some(new_time) = current_time {
            //     time.store(new_time, std::sync::atomic::Ordering::Relaxed);
            // }
            // println!(" current_time {:?}", current_time);

            let mut players_summary = Vec::new();
            let mut player_attacks_summary = Vec::new();
            let mut tile_attacks_summary = Vec::new();
            let mut players_presentation_summary = Vec::new();
            let mut tiles_summary : Vec<MapEntity>= Vec::new();
            let mut players_rewards_summary : Vec<CharacterReward>= Vec::new();
            let mut towers_summary : Vec<TowerEntity>= Vec::new();

            // let current_time = time.load(std::sync::atomic::Ordering::Relaxed);
            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            // check for delayed_commands from player
            let mut delayed_player_commands_guards = delayed_player_commands_lock.lock().await;
            let delayed_player_commands_to_execute = get_player_commands_to_execute(current_time_in_millis, &mut delayed_player_commands_guards);
            drop(delayed_player_commands_guards);

            player_commands_processor::process_delayed_player_commands(
                map.clone(), 
                &tx_pe_gameplay_longterm, 
                &mut players_summary, 
                delayed_player_commands_to_execute
            ).await;

            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            // let mut player_commands_data = player_commands_processor_lock.lock().await;
            // if player_commands_data.len() > 0  
            player_commands_processor::process_player_commands(
                map.clone(), 
                current_time_in_millis,
                player_commands_processor_lock.clone(),
                &tx_pe_gameplay_longterm, 
                &mut players_summary, 
                &mut players_presentation_summary, 
                &mut player_attacks_summary, 
                delayed_player_commands_mutex.clone()).await;

            // check for delayed_commands
            // check if tile commands can be executed.
            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            let mut delayed_tile_commands_guard = delayed_tile_commands_lock.lock().await;
            let delayed_tile_commands_to_execute = get_tile_commands_to_execute(current_time_in_millis, &mut delayed_tile_commands_guard);
            drop(delayed_tile_commands_guard);

            tile_commands_processor::process_delayed_tile_commands(
                map.clone(),
                server_state.clone(),
                &tx_me_gameplay_longterm,
                &tx_me_gameplay_webservice,
                &tx_pe_gameplay_longterm,
                &mut tiles_summary,
                &mut players_summary,
                &mut players_rewards_summary,
                delayed_tile_commands_to_execute).await;

            // drop(delayed_commands_lock);

            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            tile_commands_processor::process_tile_commands(
                map.clone(),
                server_state.clone(),
                current_time_in_millis,
                tile_commands_processor_lock.clone(),
                &tx_me_gameplay_longterm,
                &tx_me_gameplay_webservice,
                &tx_pe_gameplay_longterm,
                &mut tiles_summary,
                &mut players_summary,
                &mut players_rewards_summary,
                &mut player_attacks_summary,
                &mut tile_attacks_summary,
                delayed_tile_commands_lock.clone()).await;

            let mut delayed_tower_commands_guard = delayed_tower_commands_lock.lock().await;
            let delayed_tower_commands_to_execute = get_tower_commands_to_execute(current_time_in_millis, &mut delayed_tower_commands_guard);
            drop(delayed_tower_commands_guard);

            tower_commands_processor::process_delayed_tower_commands(
                map.clone(),
                server_state.clone(),
                &tx_te_gameplay_longterm,
                &tx_te_gameplay_webservice,
                // &tx_pe_gameplay_longterm,
                &mut towers_summary,
                &mut players_summary,
                &mut players_rewards_summary,
                delayed_tower_commands_to_execute).await;

            tower_commands_processor::process_tower_commands(
                map.clone(),
                server_state.clone(),
                tower_commands_processor_lock.clone(),
                &tx_te_gameplay_longterm,
                &tx_te_gameplay_webservice,
                // &tx_pe_gameplay_longterm,
                &mut towers_summary,
                &mut player_attacks_summary,
                delayed_tower_commands_lock.clone()).await;


            // println!("tiles summary {} ", tiles_summary.len());

            let tiles_state_update = tiles_summary
                .into_iter()
                .map(|t| StateUpdate::TileState(t));

            let tower_state_update = towers_summary
                .into_iter()
                .map(|t| StateUpdate::TowerState(t));

            let player_presentation_state_update = players_presentation_summary
                .into_iter()
                .map(|p| StateUpdate::PlayerGreetings(p));

            let player_rewards_state_update = players_rewards_summary
                .into_iter()
                .map(|p| StateUpdate::Rewards(p));

            let player_state_updates = players_summary
                .iter()
                .map(|p| StateUpdate::PlayerState(p.clone()));

            let player_attack_state_updates = player_attacks_summary
                .iter()
                .map(|p| StateUpdate::PlayerAttackState(p.clone()));

            let tile_attack_state_updates = tile_attacks_summary
                .iter()
                .map(|p| StateUpdate::TileAttackState(p.clone()));
            // Sending summary to all clients.

            let mut filtered_summary = Vec::new();


    // println!("filtered player state {}", player_state_updates.len());
            filtered_summary.extend(player_state_updates);
            filtered_summary.extend(tiles_state_update);
            filtered_summary.extend(tower_state_update);
            filtered_summary.extend(player_presentation_state_update);
            filtered_summary.extend(player_rewards_state_update);
            filtered_summary.extend(player_attack_state_updates);
            filtered_summary.extend(tile_attack_state_updates);
            // println!("filtered summarny total {}" , filtered_summary.len());
            if filtered_summary.len() > 0 
            {
                let packages = data_packer::create_data_packets(filtered_summary, &mut packet_number);
                // the data that will be sent to each client is not copied.
                let capacity = tx_bytes_game_socket.capacity();
                server_state.tx_bytes_gameplay_socket.store(capacity, std::sync::atomic::Ordering::Relaxed);
                tx_bytes_game_socket.send(packages).await.unwrap();
            }
        }
    });

    (rx_me_gameplay_longterm, rx_me_gameplay_webservice, rx_pe_gameplay_longterm, rx_te_gameplay_longterm, rx_te_gameplay_webservice, tx_mc_webservice_gameplay)
}