use std::sync::Arc;

use crate::ability_user::attack::Attack;
use crate::ability_user::attack_result::AttackResult;
use crate::character::character_presentation::CharacterPresentation;
use crate::mob::mob_command::MobCommand;
use crate::mob::mob_instance::MobEntity;
use crate::real_time_service::DataType;
use crate::ServerState;
use crate::character::character_command::{CharacterCommand, CharacterMovement};
use crate::map::GameMap;
use crate::map::map_entity::MapCommand;
use crate::character::character_entity::CharacterEntity;
use crate::character::character_reward::CharacterReward;
use crate::map::map_entity::MapEntity;
use crate::real_time_service::client_handler::StateUpdate;
use crate::tower::TowerCommand;
use crate::tower::tower_entity::TowerEntity;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use utils::get_mob_commands_to_execute;

use self::utils::{get_tile_commands_to_execute, get_player_commands_to_execute, get_tower_commands_to_execute};
pub mod data_packer;
pub mod utils;
pub mod player_commands_processor;
pub mod tile_commands_processor;
pub mod tower_commands_processor;
pub mod chat_commands_processor;
pub mod mob_commands_processor;
pub mod generic_command;


pub fn start_service(
    mut rx_pc_client_game : tokio::sync::mpsc::Receiver<CharacterCommand>,
    mut rx_mc_client_game : tokio::sync::mpsc::Receiver<MapCommand>,
    mut rx_moc_client_game : tokio::sync::mpsc::Receiver<MobCommand>,
    mut rx_tc_client_game : tokio::sync::mpsc::Receiver<TowerCommand>,
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    tx_bytes_game_socket: tokio::sync::mpsc::Sender<Vec<(u64, u8, Vec<u8>)>>
) 
-> (Receiver<MapEntity>, 
    Receiver<MapEntity>, 
    Receiver<MobEntity>, 
    Receiver<CharacterEntity>, 
    Receiver<TowerEntity>, 
    Receiver<TowerEntity>, 
    Sender<MapCommand>) 
{

    // we don't need this for the battle service because we don't store it in the db, at least for the moment.

    let (tx_mc_webservice_gameplay, mut _rx_mc_webservice_gameplay ) = tokio::sync::mpsc::channel::<MapCommand>(200);
    let (tx_me_gameplay_longterm, rx_me_gameplay_longterm ) = tokio::sync::mpsc::channel::<MapEntity>(1000);
    let (tx_me_gameplay_webservice, rx_me_gameplay_webservice) = tokio::sync::mpsc::channel::<MapEntity>(1000);
    let (tx_moe_gameplay_webservice, rx_moe_gameplay_webservice) = tokio::sync::mpsc::channel::<MobEntity>(1000);
    let (tx_pe_gameplay_longterm, rx_pe_gameplay_longterm ) = tokio::sync::mpsc::channel::<CharacterEntity>(1000);
    let (tx_te_gameplay_longterm, rx_te_gameplay_longterm ) = tokio::sync::mpsc::channel::<TowerEntity>(100);
    let (tx_te_gameplay_webservice, rx_te_gameplay_webservice) = tokio::sync::mpsc::channel::<TowerEntity>(100);

    server_state.tx_me_gameplay_longterm.store(tx_me_gameplay_longterm.capacity() as f32 as u16, std::sync::atomic::Ordering::Relaxed);
    server_state.tx_me_gameplay_webservice.store(tx_me_gameplay_webservice.capacity() as f32 as u16, std::sync::atomic::Ordering::Relaxed);
    server_state.tx_pe_gameplay_longterm.store(tx_pe_gameplay_longterm.capacity() as f32 as u16, std::sync::atomic::Ordering::Relaxed);
    server_state.tx_moe_gameplay_webservice.store(tx_moe_gameplay_webservice.capacity() as f32 as u16, std::sync::atomic::Ordering::Relaxed);

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

    //tower commands -------------------------------------
    let tower_commands = Vec::<TowerCommand>::new();
    let tower_commands_mutex = Arc::new(Mutex::new(tower_commands));
    let tower_commands_processor_lock = tower_commands_mutex.clone();
    let tower_commands_agregator_from_client_lock = tower_commands_mutex.clone();

    //mob commands -------------------------------------
    let mob_commands = Vec::<MobCommand>::new();
    let mob_commands_mutex = Arc::new(Mutex::new(mob_commands));
    let mob_commands_processor_lock = mob_commands_mutex.clone();
    let mob_commands_agregator_from_client_lock = mob_commands_mutex.clone();

    //delayed commands for attacks so they struck a bit later.
    let delayed_tile_commands = Vec::<(u64, MapCommand)>::new();
    let delayed_tile_commands_mutex = Arc::new(Mutex::new(delayed_tile_commands));
    let delayed_tile_commands_lock = delayed_tile_commands_mutex.clone();

    //delayed commands for attacks so they struck a bit later.
    let delayed_player_commands = Vec::<(u64, CharacterCommand)>::new();
    let delayed_player_commands_mutex = Arc::new(Mutex::new(delayed_player_commands));
    let delayed_player_commands_lock = delayed_player_commands_mutex.clone();

    //delayed commands for attacks so they struck a bit later.
    let delayed_tower_commands = Vec::<(u64, TowerCommand)>::new();
    let delayed_tower_commands_mutex = Arc::new(Mutex::new(delayed_tower_commands));
    let delayed_tower_commands_lock = delayed_tower_commands_mutex.clone();

    //delayed commands for attacks so they struck a bit later.
    let delayed_mob_commands = Vec::<(u64, MobCommand)>::new();
    let delayed_mob_commands_mutex = Arc::new(Mutex::new(delayed_mob_commands));
    let delayed_mob_commands_lock = delayed_mob_commands_mutex.clone();

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
        loop 
        {
            let message = rx_mc_client_game.recv().await.unwrap();
            // println!("got a tile change data {}", message.id);
            let mut data = tile_commands_agregator_from_client_lock.lock().await;
            data.push(message);
        }
    });

    tokio::spawn(async move 
    {
        // let mut sequence_number:u64 = 101;
        loop
        {
            let message = rx_tc_client_game.recv().await.unwrap();
            println!("got a tower change data {}", message.id);
            let mut data = tower_commands_agregator_from_client_lock.lock().await;
            data.push(message);
        }
    });

    tokio::spawn(async move 
    {
        loop
        {
            let message = rx_moc_client_game.recv().await.unwrap();
            let mut data = mob_commands_agregator_from_client_lock.lock().await;
            data.push(message);
        }
    });

    // task that will perdiodically send dta to all clients
    tokio::spawn(async move 
    {
        let mut buffer = [0u8; 5000];

        let mut packet_number = 1u64;
        let mut server_status_deliver_count = 0u32;

        let mut players_summary = Vec::new();
        let mut attacks_summary = Vec::new();
        let mut attack_details_summary = Vec::new();
        let mut players_presentation_summary = Vec::new();
        let mut tiles_summary : Vec<MapEntity>= Vec::new();
        let mut players_rewards_summary : Vec<CharacterReward>= Vec::new();
        let mut towers_summary : Vec<TowerEntity>= Vec::new();
        let mut mobs_summary : Vec<MobEntity>= Vec::new();

        loop 
        {
            let mut packets = Vec::new();
            interval.tick().await;

            server_status_deliver_count += 1;

            // check for delayed_commands from player
            let mut delayed_player_commands_guards = delayed_player_commands_lock.lock().await;

            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            let delayed_player_commands_to_execute = get_player_commands_to_execute(current_time_in_millis, &mut delayed_player_commands_guards);
            drop(delayed_player_commands_guards);

            player_commands_processor::process_delayed_player_commands(
                map.clone(), 
                current_time_in_millis,
                server_state.clone(),
                &tx_pe_gameplay_longterm, 
                &mut players_summary, 
                &mut attack_details_summary, 
                &mut players_rewards_summary, 
                delayed_player_commands_to_execute
            ).await;

            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            player_commands_processor::process_player_commands(
                map.clone(), 
                server_state.clone(),
                current_time_in_millis,
                player_commands_processor_lock.clone(),
                &tx_pe_gameplay_longterm, 
                &mut players_summary, 
                &mut players_presentation_summary, 
                &mut attacks_summary, 
                &mut attack_details_summary, 
                &mut players_rewards_summary, 
                delayed_player_commands_mutex.clone()).await;


            let mut delayed_tile_commands_guard = delayed_tile_commands_lock.lock().await;

            // check for delayed_commands
            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

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
                &mut attacks_summary,
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
                &mut attacks_summary,
                delayed_tower_commands_lock.clone()).await;

            let mut delayed_mob_commands_guard = delayed_mob_commands_lock.lock().await;

            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            let delayed_mob_commands_to_execute = get_mob_commands_to_execute(current_time_in_millis, &mut delayed_mob_commands_guard);
            drop(delayed_mob_commands_guard);

            mob_commands_processor::process_delayed_mob_commands(
                map.clone(),
                current_time_in_millis,
                server_state.clone(),
                &tx_moe_gameplay_webservice,
                &tx_pe_gameplay_longterm,
                &mut mobs_summary,
                &mut players_summary,
                &mut attack_details_summary,
                &mut players_rewards_summary,
                delayed_mob_commands_to_execute).await;

            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            mob_commands_processor::process_mob_commands(
                map.clone(),
                current_time_in_millis,
                server_state.clone(),
                &tx_pe_gameplay_longterm,
                &tx_moe_gameplay_webservice,
                mob_commands_processor_lock.clone(),
                delayed_mob_commands_lock.clone(),
                &mut mobs_summary,
                &mut players_summary,
                &mut attack_details_summary,
                &mut players_rewards_summary,
                &mut attacks_summary,
                ).await;

            let mut offset : usize;
            offset = data_packer::init_data_packet(&mut buffer, &mut packet_number);

            let len = tiles_summary.len();
            if len > 0
            {
                println!("--tiles {len}");
            }
            tiles_summary.drain(..)
            .for_each(|d| 
            {
                let chunk = d.to_bytes();
                let chunk_size = MapEntity::get_size();
                data_packer::build_data_packet(
                    &mut packet_number,
                    &mut buffer,
                    &mut packets,
                    &mut offset,
                    DataType:: TileState,
                    &chunk,
                    chunk_size);
            });

            towers_summary.drain(..)
            .for_each(|d| 
            {
                let chunk = d.to_bytes();
                let chunk_size = TowerEntity::get_size();
                data_packer::build_data_packet(
                    &mut packet_number,
                    &mut buffer,
                    &mut packets,
                    &mut offset,
                    DataType::TowerState,
                    &chunk,
                    chunk_size);
            });

            players_presentation_summary.drain(..)
            .for_each(|d| 
            {
                let chunk = d.to_bytes();
                let chunk_size = CharacterPresentation::get_size();
                data_packer::build_data_packet(
                    &mut packet_number,
                    &mut buffer,
                    &mut packets,
                    &mut offset,
                    DataType::PlayerPresentation,
                    &chunk,
                    chunk_size);
            });

            players_rewards_summary.drain(..)
            .for_each(|d| 
            {
                let chunk = d.to_bytes();
                let chunk_size = CharacterReward::get_size();
                data_packer::build_data_packet(
                    &mut packet_number,
                    &mut buffer,
                    &mut packets,
                    &mut offset,
                    DataType::PlayerReward,
                    &chunk,
                    chunk_size);
            });

            let len = players_summary.len();
            if len > 0
            {
                println!("--players {len}");
            }
            players_summary.drain(..)
            .for_each(|d| 
            {
                let chunk = d.to_bytes();
                let chunk_size = CharacterEntity::get_size();
                data_packer::build_data_packet(
                    &mut packet_number,
                    &mut buffer,
                    &mut packets,
                    &mut offset,
                    DataType::PlayerState,
                    &chunk,
                    chunk_size);
            });

            let len = attacks_summary.len();
            if len > 0
            {
                println!("--attacks {len}");
            }
            attacks_summary.drain(..)
            .for_each(|d| 
            {
                let a = d.battle_type;
                println!("--- sending an attack summary {a}");
                let chunk = d.to_bytes();
                let chunk_size = Attack::get_size();
                data_packer::build_data_packet(
                    &mut packet_number,
                    &mut buffer,
                    &mut packets,
                    &mut offset,
                    DataType::Attack,
                    &chunk,
                    chunk_size);
            });

            let len = attack_details_summary.len();
            if len > 0
            {
                println!("--results {len}");
            }
            attack_details_summary.drain(..)
            .for_each(|d| 
            {
                let chunk = d.to_bytes();
                let chunk_size = AttackResult::get_size();
                data_packer::build_data_packet(
                    &mut packet_number,
                    &mut buffer,
                    &mut packets,
                    &mut offset,
                    DataType::AttackDetails,
                    &chunk,
                    chunk_size);
            });

            let len = mobs_summary.len();
            if len > 0
            {
                println!("--mobs {len}");
            }
            mobs_summary.drain(..)
            .for_each(|d| 
            {
                let chunk = d.to_bytes();
                let chunk_size = MobEntity::get_size();
                data_packer::build_data_packet(
                    &mut packet_number,
                    &mut buffer,
                    &mut packets,
                    &mut offset,
                    DataType::MobStatus,
                    &chunk,
                    chunk_size);
            });

            server_status_deliver_count += packets.len() as u32;
            if server_status_deliver_count > 50
            {
                server_status_deliver_count = 0;

                let stats = server_state.get_stats();
                let chunk = ServerState::stats_to_bytes(&stats); //20
                let chunk_size = ServerState::get_size();
                data_packer::build_data_packet(
                    &mut packet_number,
                    &mut buffer,
                    &mut packets,
                    &mut offset,
                    DataType::ServerStatus,
                    &chunk,
                    chunk_size);
            }

            if offset > 0
            {
                let encoded_data = data_packer::encode_packet(&mut buffer, offset);
                packets.push((packet_number, 0, encoded_data));
            }

            if packets.len() > 0 
            {
                let capacity = tx_bytes_game_socket.capacity();
                server_state.tx_bytes_gameplay_socket.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
                tx_bytes_game_socket.send(packets).await.unwrap();
            }
        }
    });

    (
        rx_me_gameplay_longterm,
        rx_me_gameplay_webservice,
        rx_moe_gameplay_webservice,
        rx_pe_gameplay_longterm,
        rx_te_gameplay_longterm,
        rx_te_gameplay_webservice,
        tx_mc_webservice_gameplay
    )
}