use std::sync::Arc;

use crate::real_time_service::client_handler::StateUpdate;

pub struct PlayerEntity {
    pub player_id: u64,
    pub tx: tokio::sync::mpsc::Sender<Arc<Vec<StateUpdate>>>
}

// impl PlayerState {
//     pub fn to_bytes(&self) -> [u8;36] {
//     }
// }