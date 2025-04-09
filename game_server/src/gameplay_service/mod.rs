use std::sync::Arc;
use std::vec;

use crate::ability_user::attack::Attack;
use crate::ability_user::attack_result::AttackResult;
use crate::hero::hero_presentation::HeroPresentation;
use crate::gaia_mpsc::GaiaSender;
use crate::mob::mob_command::MobCommand;
use crate::mob::mob_instance::MobEntity;
use crate::clients_service::DataType;
use crate::{gaia_mpsc, ServerChannels, ServerState};
use crate::hero::hero_command::{HeroCommand, HeroMovement};
use crate::map::GameMap;
use crate::map::map_entity::MapCommand;
use crate::hero::hero_entity::HeroEntity;
use crate::hero::hero_reward::HeroReward;
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
pub mod hero_commands_processor;
pub mod tile_commands_processor;
pub mod tower_commands_processor;
pub mod chat_commands_processor;
pub mod mob_commands_processor;
pub mod generic_command;

pub struct PacketsData
{
    started:bool,
    region:u16,
    packet_number: u64,
    packets: Vec<(u64, u8, u16, u32, Vec<u8>)>,
    buffer: [u8;5000],
    game_packets_count : u32,
    offset : usize,
}

pub fn start_service(
    mut rx_hc_client_game : tokio::sync::mpsc::Receiver<HeroCommand>,
    mut rx_mc_client_game : tokio::sync::mpsc::Receiver<MapCommand>,
    mut rx_moc_client_game : tokio::sync::mpsc::Receiver<MobCommand>,
    mut rx_tc_client_game : tokio::sync::mpsc::Receiver<TowerCommand>,
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    tx_bytes_game_socket: gaia_mpsc::GaiaSender<Vec<(u64, u8, u16, u32, Vec<u8>)>>
) 
-> (Receiver<MapEntity>, 
    Receiver<MapEntity>, 
    Receiver<MobEntity>, 
    Receiver<HeroEntity>, 
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
    let (tx_he_gameplay_longterm, rx_he_gameplay_longterm ) = gaia_mpsc::channel::<HeroEntity>(100, ServerChannels::TX_PE_GAMEPLAY_LONGTERM, server_state.clone());
    let (tx_te_gameplay_longterm, rx_te_gameplay_longterm ) = gaia_mpsc::channel::<TowerEntity>(100, ServerChannels::TX_TE_GAMEPLAY_LONGTERM, server_state.clone());
    let (tx_te_gameplay_webservice, rx_te_gameplay_webservice) = gaia_mpsc::channel::<TowerEntity>(100, ServerChannels::TX_TE_GAMEPLAY_WEBSERVICE, server_state.clone());

    //players
    //player commands -------------------------------------
    let player_commands = Vec::<HeroCommand>::new();
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
    let delayed_player_commands = Vec::<(u64, HeroCommand)>::new();
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
    tokio::spawn(async move 
    {
        loop 
        {
            let message = rx_hc_client_game.recv().await.unwrap();

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
    tokio::spawn(async move 
    {
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
        let mut server_status_deliver_count = 0u32;

        let mut heroes_summary = Vec::new();
        let mut attacks_summary = Vec::new();
        let mut attack_details_summary = Vec::new();
        let mut heroes_presentation_summary = Vec::new();
        let mut tiles_summary : Vec<MapEntity>= Vec::new();
        let mut heroes_rewards_summary : Vec<HeroReward>= Vec::new();
        let mut towers_summary : Vec<TowerEntity>= Vec::new();
        let mut mobs_summary : Vec<MobEntity>= Vec::new();

        let mut previous_time : u64 = 0;

        let mut packets_data : Vec<PacketsData> = Vec::new();             
        for (i, region_id) in map.definitions.regions_by_id.iter().enumerate()
        {
            packets_data.push(PacketsData 
            {
                started: false,
                region: i as u16,
                packet_number: 0,
                packets: Vec::new(),
                buffer: [0u8; 5000],
                game_packets_count: 0,
                offset: 0,
            });
        }

        loop 
        {
            // let mut packets = Vec::new();
            interval.tick().await;

            server_status_deliver_count += 1;

            // check for delayed_commands from player
            let mut delayed_player_commands_guards = delayed_player_commands_lock.lock().await;

            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            let delayed_player_commands_to_execute = get_player_commands_to_execute(current_time_in_millis, &mut delayed_player_commands_guards);
            drop(delayed_player_commands_guards);

            hero_commands_processor::process_delayed_hero_commands(
                map.clone(), 
                current_time_in_millis,
                server_state.clone(),
                &tx_he_gameplay_longterm, 
                &mut heroes_summary, 
                &mut attack_details_summary, 
                &mut heroes_rewards_summary, 
                delayed_player_commands_to_execute
            ).await;

            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            hero_commands_processor::process_hero_commands(
                map.clone(), 
                server_state.clone(),
                current_time_in_millis,
                player_commands_processor_lock.clone(),
                &tx_he_gameplay_longterm, 
                &mut heroes_summary, 
                &mut heroes_presentation_summary, 
                &mut attacks_summary, 
                &mut attack_details_summary, 
                &mut heroes_rewards_summary, 
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
                &tx_he_gameplay_longterm,
                &mut tiles_summary,
                &mut heroes_summary,
                &mut heroes_rewards_summary,
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
                &tx_he_gameplay_longterm,
                &mut tiles_summary,
                &mut heroes_summary,
                &mut heroes_rewards_summary,
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
                &mut heroes_summary,
                &mut heroes_rewards_summary,
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
                &tx_he_gameplay_longterm,
                &mut mobs_summary,
                &mut heroes_summary,
                &mut attack_details_summary,
                &mut heroes_rewards_summary,
                delayed_mob_commands_to_execute).await;

            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            mob_commands_processor::process_mob_commands(
                map.clone(),
                current_time_in_millis,
                server_state.clone(),
                &tx_he_gameplay_longterm,
                &tx_moe_gameplay_webservice,
                mob_commands_processor_lock.clone(),
                delayed_mob_commands_lock.clone(),
                &mut mobs_summary,
                &mut heroes_summary,
                &mut attack_details_summary,
                &mut heroes_rewards_summary,
                &mut attacks_summary,
                ).await;


            let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
            let current_time_in_millis = current_time.as_millis() as u64;

            let game_packages= 
                tiles_summary.len() +
                towers_summary.len() +
                heroes_presentation_summary.len() +
                heroes_rewards_summary.len() +
                heroes_summary.len() +
                attacks_summary.len() +
                attack_details_summary.len() +
                mobs_summary.len();

            // if game_packages == 0 && (current_time_in_millis - previous_time) < 1000
            // {
            //     // cli_log::info!("--- skipping, no data to transmit");
            //     continue;
            // }



            // let len = tiles_summary.len();
            // if len > 0
            // {
            //     cli_log::info!("--tiles {len}");
            // }
            tiles_summary.drain(..)
            .for_each(|d| 
            {
                let region = map.definitions.regions_by_id.get(&d.id.get_parent(7)).unwrap();
                let mut region_packets_data = packets_data.get_mut(*region as usize).unwrap();
                let chunk = d.to_bytes();
                let chunk_size = MapEntity::get_size();
                data_packer::build_data_packet(
                    &mut region_packets_data,
                    DataType:: TileState,
                    &chunk,
                    chunk_size);
            });

            towers_summary.drain(..)
            .for_each(|d| 
            {
                let region = map.definitions.regions_by_id.get(&d.tetrahedron_id.get_parent(7)).unwrap();
                let mut region_packets_data = packets_data.get_mut(*region as usize).unwrap();
                let chunk = d.to_bytes();
                let chunk_size = TowerEntity::get_size();
                data_packer::build_data_packet(
                    &mut region_packets_data,
                    DataType::TowerState,
                    &chunk,
                    chunk_size);
            });

            heroes_presentation_summary.drain(..)
            .for_each(|d| 
            {
                // presentation goes to everyone...
                let mut region_packets_data = packets_data.get_mut(0).unwrap();
                let chunk = d.to_bytes();
                let chunk_size = HeroPresentation::get_size();
                data_packer::build_data_packet(
                    &mut region_packets_data,
                    DataType::PlayerPresentation,
                    &chunk,
                    chunk_size);
            });

            heroes_rewards_summary.drain(..)
            .for_each(|d| 
            {
                let mut region_packets_data = packets_data.get_mut(0).unwrap();
                let chunk = d.to_bytes();
                let chunk_size = HeroReward::get_size();
                data_packer::build_data_packet(
                    &mut region_packets_data,
                    DataType::PlayerReward,
                    &chunk,
                    chunk_size);
            });

            let len = heroes_summary.len();
            if len > 0
            {
                cli_log::info!("--players {len}");
            }
            heroes_summary.drain(..)
            .for_each(|d| 
            {
                let region = map.definitions.regions_by_id.get(&d.position.get_parent(7)).unwrap();
                let mut region_packets_data = packets_data.get_mut(*region as usize).unwrap();
                let chunk = d.to_bytes();
                let chunk_size = HeroEntity::get_size();
                data_packer::build_data_packet(
                    &mut region_packets_data,
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
                let mut region_packets_data = packets_data.get_mut(0).unwrap();
                let a = d.battle_type;
                cli_log::info!("--- sending an attack summary {a}");
                let chunk = d.to_bytes();
                let chunk_size = Attack::get_size();
                data_packer::build_data_packet(
                    region_packets_data,
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
                let mut region_packets_data = packets_data.get_mut(0).unwrap();
                let chunk = d.to_bytes();
                let chunk_size = AttackResult::get_size();
                data_packer::build_data_packet(
                    &mut region_packets_data,
                    DataType::AttackDetails,
                    &chunk,
                    chunk_size);
            });

            // let len = mobs_summary.len();
            // if len > 0
            // {
            //     cli_log::info!("--mobs {len}");
            // }
            mobs_summary.drain(..)
            .for_each(|d| 
            {
                let region = map.definitions.regions_by_id.get(&d.tile_id.get_parent(7)).unwrap();
                let mut region_packets_data = packets_data.get_mut(*region as usize).unwrap();
                let chunk = d.to_bytes();
                let chunk_size = MobEntity::get_size();
                data_packer::build_data_packet(
                    &mut region_packets_data,
                    DataType::MobStatus,
                    &chunk,
                    chunk_size);
            });

            let mut global_regions_packets_data = packets_data.get_mut(0).unwrap();
            server_status_deliver_count += global_regions_packets_data.packets.len() as u32;
            if server_status_deliver_count > 10 
            {
                server_status_deliver_count = 0;

                let stats = server_state.get_stats();
                let chunk = ServerState::stats_to_bytes(&stats); //20
                let chunk_size = ServerState::get_size();
                data_packer::build_data_packet(
                    &mut global_regions_packets_data,
                    DataType::ServerStatus,
                    &chunk,
                    chunk_size);

                // let time_since_last_message =  current_time_in_millis - previous_time;
                // cli_log::info!("---transmit since last status {}",time_since_last_message);
                previous_time = current_time_in_millis;
            }

            // 10 milliseconds * 50 = 500 milliseconds



            // cli_log::info!("---checking regions data packets");
            for region_packets_data in packets_data.iter_mut()
            {
                if region_packets_data.offset > 0
                {
                    let encoded_data = data_packer::encode_packet(&mut region_packets_data.buffer, region_packets_data.offset);
                    region_packets_data.packets.push((region_packets_data.packet_number, 0, region_packets_data.region, region_packets_data.game_packets_count, encoded_data));
                }

                if region_packets_data.packets.len() > 0 
                {
                    // cli_log::info!("--found packets for region {} size: {} game_packets: {}", region_packets_data.region, region_packets_data.packets.iter().len(), region_packets_data.game_packets_count);
                    let mut temp_vec = Vec::new();
                    std::mem::swap(&mut region_packets_data.packets, &mut temp_vec);
                    tx_bytes_game_socket.send(temp_vec).await.unwrap();
                    region_packets_data.started = false;
                    region_packets_data.offset = 0;
                }
            }


        }
    });

    (
        rx_me_gameplay_longterm,
        rx_me_gameplay_webservice,
        rx_moe_gameplay_webservice,
        rx_he_gameplay_longterm,
        rx_te_gameplay_longterm,
        rx_te_gameplay_webservice,
        tx_mc_webservice_gameplay
    )
}