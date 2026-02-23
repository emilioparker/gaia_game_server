use std::sync::Arc;
use tokio::sync::Mutex;
use crate::ability_user::attack::Attack;
use crate::gaia_mpsc::GaiaSender;
use crate::kingdom::kingdom_entity::KingdomEntity;
use crate::kingdom::KingdomCommand;
use crate::map::GameMap;
use crate::ServerState;


pub async fn process_kingdoms_commands (
    _map : Arc<GameMap>,
    _server_state: Arc<ServerState>,
    kingdoms_commands_processor_lock : Arc<Mutex<Vec<KingdomCommand>>>,
    _tx_ke_gameplay_longterm : &GaiaSender<KingdomEntity>,
    _tx_ke_gameplay_webservice : &GaiaSender<KingdomEntity>,
    _kingdoms_summary : &mut Vec<KingdomEntity>,
    _player_attacks_summary : &mut  Vec<Attack>,
    // delayed_tower_commands_lock : Arc<Mutex<Vec<(u64, TowerCommand)>>>
)
{
    // process tower stuff.
    let kingdom_commands_data = kingdoms_commands_processor_lock.lock().await;
    // cli_log::info!("tower commands len {}", tower_commands_data.len());
    if kingdom_commands_data.len() > 0 
    {
        for _kingdom_command in kingdom_commands_data.iter()
        {
        }
    }
}