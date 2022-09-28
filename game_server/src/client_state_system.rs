use std::{sync::Arc, borrow::Borrow, time::SystemTime};

use crate::{player_state::PlayerState, player_action::PlayerAction, player_entity::PlayerEntity, tetrahedron_id::TetrahedronId};
use tokio::{sync::Mutex, select};
use std::collections::HashMap;



pub fn process_player_action(
    mut action_receiver : tokio::sync::mpsc::Receiver<PlayerAction>,
    players : Arc<Mutex<HashMap<std::net::SocketAddr,PlayerEntity>>>){

    //players
    let all_players = HashMap::<u64,PlayerState>::new();
    let data_mutex = Arc::new(Mutex::new(all_players));
    let processor_lock = data_mutex.clone();
    let agregator_lock = data_mutex.clone();

    // tiles
    let all_tiles = HashMap::<TetrahedronId,u32>::new();
    let tiles_mutex = Arc::new(Mutex::new(all_tiles));
    let tiles_processor_lock = tiles_mutex.clone();
    let tiles_agregator_lock = tiles_mutex.clone();

    let mut seq = 0;

    //task that will handle receiving and updating the state.
    tokio::spawn(async move {

        let mut sequence_number:u64 = 101;
        loop {
            let message = action_receiver.recv().await.unwrap();

            let mut current_time = 0;
            let result = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
            if let Ok(elapsed) = result {
                current_time = elapsed.as_secs();
            }


            sequence_number = sequence_number + 1;

            let mut data = agregator_lock.lock().await;
            
            // here we have access to the players data;
            match message 
            {
                PlayerAction::Activity(activity) =>
                {
                    let new_client_state = PlayerState{
                        current_time,
                        sequence_number,
                        player_id : activity.player_id,
                        position : activity.position,
                        second_position : activity.direction,
                        action : activity.action
                    };

                    // println!("player {} pos {:?}",seq, message.position);
                    seq = seq + 1;

                    let old = data.get(&activity.player_id);
                    match old {
                        Some(_previous_record) => {
                            data.insert(activity.player_id, new_client_state);
                        }
                        _ => {
                            data.insert(activity.player_id, new_client_state);
                        }
                    }
                },
                PlayerAction::Interaction(_) => {

                }
            }

        }
    });


    tokio::spawn(async move {
        let mut players_summary = Vec::new();
        loop {
            // assuming 30 fps.
            tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;

            let mut data = processor_lock.lock().await;
            if data.len() <= 0 {
                continue;
            }

            let mut max_seq = 0;

            for item in data.iter()
            {
                let cloned_data = item.1.to_owned();
                players_summary.push(cloned_data);
                max_seq = std::cmp::max(max_seq, item.1.borrow().sequence_number);
            }
            // we should easily get this lock, since only new clients would trigger a lock on the other side.
            let mut clients_data = players.lock().await;

            for client in clients_data.iter_mut()
            {
                let filtered_summary = players_summary.iter().filter(|p| {
                    p.sequence_number > client.1.sequence_number
                })
                .map(|p| p.clone())
                .collect();
                client.1.tx.send(filtered_summary).await.unwrap();
                client.1.sequence_number = max_seq;
            }


            players_summary.clear();

            let result = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
            if let Ok(elapsed) = result {
                let current_time = elapsed.as_secs();
                data.retain(|_, v| (current_time - v.current_time) < 20);
            }
        }
    });
}