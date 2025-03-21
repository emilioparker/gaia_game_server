use std::sync::Arc;

use crate::ability_user::attack::Attack;
use crate::ability_user::attack_result::AttackResult;
use crate::character::character_presentation::CharacterPresentation;
use crate::gaia_mpsc::GaiaSender;
use crate::mob::mob_command::MobCommand;
use crate::mob::mob_instance::MobEntity;
use crate::clients_service::DataType;
use crate::{gaia_mpsc, ServerChannels, ServerState};
use crate::character::character_command::{CharacterCommand, CharacterMovement};
use crate::map::GameMap;
use crate::map::map_entity::MapCommand;
use crate::character::character_entity::CharacterEntity;
use crate::character::character_reward::CharacterReward;
use crate::map::map_entity::MapEntity;
use crate::clients_service::client_handler::StateUpdate;
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
    tx_bytes_game_socket: gaia_mpsc::GaiaSender<Vec<(u64, u8, u32, Vec<u8>)>>
) 
-> (Receiver<MapEntity>, 
    Receiver<MapEntity>, 
    Receiver<MobEntity>, 
    Receiver<CharacterEntity>, 
    Receiver<TowerEntity>, 
    Receiver<TowerEntity>, 
    GaiaSender<MapCommand>) 
{

    // we don't need this for the battle service because we don't store it in the db, at least for the moment.

    // we can receive map commands from the web client.. or at least used to be able.
    let (tx_mc_webservice_gameplay, mut _rx_mc_webservice_gameplay ) = gaia_mpsc::channel::<MapCommand>(100, ServerChannels::TX_MC_WEBSERVICE_GAMEPLAY, server_state.clone());
    let (tx_me_gameplay_longterm, rx_me_gameplay_longterm ) = gaia_mpsc::channel::<MapEntity>(100, ServerChannels::TX_ME_GAMEPLAY_LONGTERM, server_state.clone());
    let (tx_me_gameplay_webservice, rx_me_gameplay_webservice) = gaia_mpsc::channel::<MapEntity>(100, ServerChannels::TX_ME_GAMEPLAY_WEBSERVICE, server_state.clone());
    let (tx_moe_gameplay_webservice, rx_moe_gameplay_webservice) = gaia_mpsc::channel::<MobEntity>(100, ServerChannels::TX_MOE_GAMEPLAY_WEBSERVICE, server_state.clone());
    let (tx_pe_gameplay_longterm, rx_pe_gameplay_longterm ) = gaia_mpsc::channel::<CharacterEntity>(100, ServerChannels::TX_PE_GAMEPLAY_LONGTERM, server_state.clone());
    let (tx_te_gameplay_longterm, rx_te_gameplay_longterm ) = gaia_mpsc::channel::<TowerEntity>(100, ServerChannels::TX_TE_GAMEPLAY_LONGTERM, server_state.clone());
    let (tx_te_gameplay_webservice, rx_te_gameplay_webservice) = gaia_mpsc::channel::<TowerEntity>(100, ServerChannels::TX_TE_GAMEPLAY_WEBSERVICE, server_state.clone());

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

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(10));

    //task that will handle receiving state changes from clients and updating the global statestate.
    tokio::spawn(async move {

        loop {
            let message = rx_pc_client_game.recv().await.unwrap();

            // cli_log::info!("got a player change data {}", message.player_id);
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
            // cli_log::info!("got a tile change data {}", message.id);
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
            cli_log::info!("got a tower change data {}", message.id);
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

        let mut previous_time : u64 = 0;

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


            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            let game_packages= 
                tiles_summary.len() +
                towers_summary.len() +
                players_presentation_summary.len() +
                players_rewards_summary.len() +
                players_summary.len() +
                attacks_summary.len() +
                attack_details_summary.len() +
                mobs_summary.len();

            if game_packages == 0 && (current_time_in_millis - previous_time) < 1000
            {
                // cli_log::info!("--- skipping");
                continue;
            }

            previous_time = current_time_in_millis;

            let mut game_packets_count : u32 = 0;
            let mut offset : usize;
            offset = data_packer::init_data_packet(&mut buffer, &mut packet_number);

            let len = tiles_summary.len();
            if len > 0
            {
                cli_log::info!("--tiles {len}");
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
                    &mut game_packets_count,
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
                    &mut game_packets_count,
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
                    &mut game_packets_count,
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
                    &mut game_packets_count,
                    DataType::PlayerReward,
                    &chunk,
                    chunk_size);
            });

            let len = players_summary.len();
            if len > 0
            {
                cli_log::info!("--players {len}");
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
                    &mut game_packets_count,
                    DataType::PlayerState,
                    &chunk,
                    chunk_size);
            });

            let len = attacks_summary.len();
            if len > 0
            {
                cli_log::info!("--attacks {len}");
            }
            attacks_summary.drain(..)
            .for_each(|d| 
            {
                let a = d.battle_type;
                cli_log::info!("--- sending an attack summary {a}");
                let chunk = d.to_bytes();
                let chunk_size = Attack::get_size();
                data_packer::build_data_packet(
                    &mut packet_number,
                    &mut buffer,
                    &mut packets,
                    &mut offset,
                    &mut game_packets_count,
                    DataType::Attack,
                    &chunk,
                    chunk_size);
            });

            let len = attack_details_summary.len();
            if len > 0
            {
                cli_log::info!("--results {len}");
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
                    &mut game_packets_count,
                    DataType::AttackDetails,
                    &chunk,
                    chunk_size);
            });

            let len = mobs_summary.len();
            if len > 0
            {
                cli_log::info!("--mobs {len}");
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
                    &mut game_packets_count,
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
                    &mut game_packets_count,
                    DataType::ServerStatus,
                    &chunk,
                    chunk_size);
            }

            if offset > 0
            {
                let encoded_data = data_packer::encode_packet(&mut buffer, offset);
                packets.push((packet_number, 0, game_packets_count, encoded_data));
            }

            if packets.len() > 0 
            {
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