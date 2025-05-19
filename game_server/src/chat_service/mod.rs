use std::sync::Arc;

use crate::{gaia_mpsc, ServerState};
use crate::chat::ChatCommand;
use crate::chat::chat_entry::ChatEntry;
use crate::map::GameMap;
use bytes::Bytes;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;

pub mod chat_data_packer;
pub mod chat_commands_processor;

pub fn start_service(
    mut rx_cc_client_game : tokio::sync::mpsc::Receiver<ChatCommand>,
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    tx_packets_gameplay_chat_clients: gaia_mpsc::GaiaSender<Vec<(u64, u8, u16, u32, Bytes)>> //faction-data 0 means global
) 
-> Receiver<ChatEntry>
{

    let (tx_ce_chat_webservice, rx_ce_chat_webservice) = gaia_mpsc::channel::<ChatEntry>(100, crate::ServerChannels::TX_CE_CHAT_WEBSERVICE, server_state.clone());

    //message commands -------------------------------------
    let chat_commands = Vec::<ChatCommand>::new();
    let chat_commands_mutex = Arc::new(Mutex::new(chat_commands));
    let chat_commands_processor_lock = chat_commands_mutex.clone();
    let chat_commands_agregator_from_client_lock = chat_commands_mutex.clone();

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));

    tokio::spawn(async move 
    {
        loop 
        {
            let message = rx_cc_client_game.recv().await.unwrap();
            cli_log::info!("got a message data {}", message.id);
            let mut data = chat_commands_agregator_from_client_lock.lock().await;
            data.push(message);
        }
    });

    // task that will perdiodically send dta to all clients
    tokio::spawn(async move 
    {
        // by faction, but we only have 3...
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
                &tx_ce_chat_webservice,
                &mut chat_summary,
                ).await;

            // cli_log::info!("filtered summarny total {}" , filtered_summary.len());
            // separar por faccion.
            //empaquetar mensajes por faccion.

            for (faction, faction_chat) in chat_summary.iter().enumerate()
            {
                if faction_chat.len() > 0 
                {
                    let packages = chat_data_packer::create_data_packets(faction as u8, faction_chat, &mut packet_number);
                    // the data that will be sent to each client is not copied.
                    tx_packets_gameplay_chat_clients.send(packages).await.unwrap();
                }
            }

            for faction_summary in &mut chat_summary
            {
                faction_summary.clear();
            }
        }
    });

    rx_ce_chat_webservice
}