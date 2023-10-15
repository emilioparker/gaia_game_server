use std::sync::Arc;

use crate::ServerState;
use crate::chat::ChatCommand;
use crate::chat::chat_entry::ChatEntry;
use crate::map::GameMap;
use crate::real_time_service::client_handler::StateUpdate;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

pub mod chat_data_packer;
pub mod chat_commands_processor;

// pub enum DataType
// {
//     NoData = 25,
//     PlayerState = 26,
//     TileState = 27,
//     PlayerPresentation = 28,
//     PlayerAttack = 29,
//     PlayerReward = 30,
//     TileAttack = 31,
//     TowerState = 32,
//     ChatMessage = 33,
// }

pub fn start_service(
    mut rx_cc_client_game : tokio::sync::mpsc::Receiver<ChatCommand>,
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    tx_bytes_game_socket: tokio::sync::mpsc::Sender<Vec<(u64, u8, Vec<u8>)>> //faction-data 0 means global
) 
-> Receiver<ChatEntry>
{

    let (tx_ce_gameplay_webservice, rx_ce_gameplay_webservice) = tokio::sync::mpsc::channel::<ChatEntry>(100);

    //message commands -------------------------------------
    let chat_commands = Vec::<ChatCommand>::new();
    let chat_commands_mutex = Arc::new(Mutex::new(chat_commands));
    let chat_commands_processor_lock = chat_commands_mutex.clone();
    let chat_commands_agregator_from_client_lock = chat_commands_mutex.clone();

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(1000));

    tokio::spawn(async move 
    {
        loop 
        {
            let message = rx_cc_client_game.recv().await.unwrap();
            println!("got a message data {}", message.id);
            let mut data = chat_commands_agregator_from_client_lock.lock().await;
            data.push(message);
        }
    });

    // task that will perdiodically send dta to all clients
    tokio::spawn(async move 
    {
        let mut chat_summary : [Vec<ChatEntry>; 10] = 
        [
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ];

        let mut packet_number = 1u64;
        loop 
        {
            interval.tick().await;


            chat_commands_processor::process_chat_commands(
                map.clone(),
                server_state.clone(),
                chat_commands_processor_lock.clone(),
                &tx_ce_gameplay_webservice,
                &mut chat_summary,
                ).await;

            // println!("filtered summarny total {}" , filtered_summary.len());
            // separar por faccion.
            //empaquetar mensajes por faccion.

            for (faction, faction_summary) in chat_summary.iter().enumerate()
            {
                if faction_summary.len() > 0 
                {
                    let packages = chat_data_packer::create_data_packets(faction as u8, faction_summary, &mut packet_number);
                    // the data that will be sent to each client is not copied.
                    let capacity = tx_bytes_game_socket.capacity();
                    server_state.tx_bytes_gameplay_socket.store(capacity as f32 as u16, std::sync::atomic::Ordering::Relaxed);
                    tx_bytes_game_socket.send(packages).await.unwrap();
                }
            }

            for faction_summary in &mut chat_summary
            {
                faction_summary.clear();
            }
        }
    });

    rx_ce_gameplay_webservice
}