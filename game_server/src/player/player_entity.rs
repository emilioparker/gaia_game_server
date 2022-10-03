use crate::real_time_service::client_handler::StateUpdate;

pub struct PlayerEntity {
    pub sequence_number: u64,
    pub player_id: u64,
    pub tx: tokio::sync::mpsc::Sender<Vec<StateUpdate>>
}

// impl PlayerState {
//     pub fn to_bytes(&self) -> [u8;36] {
//     }
// }