use std::{sync::Arc, collections::HashMap};
use tokio::sync::{mpsc::Sender, Mutex};

use crate::{chat::{ChatCommand, chat_entry::ChatEntry}, map::GameMap, ServerState};


pub async fn process_chat_commands (
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    chat_commands_processor_lock : Arc<Mutex<Vec<ChatCommand>>>,
    // tx_te_gameplay_webservice : &Sender<TowerEntity>,
    chat_summary : &mut Vec<ChatEntry>,
)
{
    // process tower stuff.
    let mut chat_commands_data = chat_commands_processor_lock.lock().await;
    // println!("tower commands len {}", tower_commands_data.len());
    if chat_commands_data.len() > 0 
    {
        for chat_command in chat_commands_data.iter()
        {
            chat_summary.push(ChatEntry 
            { 
                tetrahedron_id: chat_command.id.clone(),
                faction: chat_command.faction,
                player_id: chat_command.player_id,
                message_length: chat_command.message_length,
                message: chat_command.message 
            });
            println!("added chat entry");
        }
    }
    chat_commands_data.clear();
}