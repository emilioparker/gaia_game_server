use std::sync::Arc;
use tokio::sync::{mpsc::Sender, Mutex};

use crate::{chat::{ChatCommand, chat_entry::ChatEntry}, map::GameMap, ServerState};


pub async fn process_chat_commands (
    _map : Arc<GameMap>,
    _server_state: Arc<ServerState>,
    chat_commands_processor_lock : Arc<Mutex<Vec<ChatCommand>>>,
    tx_ce_gameplay_webservice : &Sender<ChatEntry>,
    chat_summary : &mut [Vec<ChatEntry>; 10],
)
{
    let mut chat_commands_data = chat_commands_processor_lock.lock().await;
    let current_time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
    let current_time_in_seconds = current_time.as_secs() as u32;
    if chat_commands_data.len() > 0 
    {
        for chat_command in chat_commands_data.iter()
        {
            let chat_entry = ChatEntry 
            { 
                tetrahedron_id: chat_command.id.clone(),
                timestamp: current_time_in_seconds,
                faction: chat_command.faction,
                player_id: chat_command.player_id,
                message_length: chat_command.message_length,
                message: chat_command.message 
            };
            let _send_result = tx_ce_gameplay_webservice.send(chat_entry.clone()).await;
            chat_summary[chat_command.faction as usize].push(chat_entry);
            cli_log::info!("added chat entry");
        }
    }
    chat_commands_data.clear();
}