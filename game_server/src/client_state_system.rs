use std::mem;
use std::{sync::Arc, borrow::Borrow, time::SystemTime};

use crate::player::{player_state::PlayerState, player_action::PlayerAction, player_entity::PlayerEntity};
use crate::map::{tetrahedron_id::TetrahedronId, map_entity::MapEntity};
use crate::real_time_service::client_handler::StateUpdate;
use tokio::{sync::Mutex};
use std::collections::HashMap;


pub fn process_player_action(
    mut action_receiver : tokio::sync::mpsc::Receiver<PlayerAction>,
    mut tile_changed_receiver : tokio::sync::mpsc::Receiver<MapEntity>,
    players : Arc<Mutex<HashMap<std::net::SocketAddr,PlayerEntity>>>){

    //players
    let all_players = HashMap::<u64,PlayerState>::new();
    let data_mutex = Arc::new(Mutex::new(all_players));
    let processor_lock = data_mutex.clone();
    let agregator_lock = data_mutex.clone();

    //tiles
    let modified_tiles = HashMap::<TetrahedronId,MapEntity>::new();
    let tiles_data_mutex = Arc::new(Mutex::new(modified_tiles));

    let tiles_processor_lock = tiles_data_mutex.clone();
    let tiles_agregator_lock = tiles_data_mutex.clone();

    let mut seq = 0;

    //task that will handle receiving state changes from clients and updating the global statestate.
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

    // task that gathers world changes into a list.
    tokio::spawn(async move {

        let mut sequence_number:u64 = 101;
        loop {
            let message = tile_changed_receiver.recv().await.unwrap();
            // println!("got a tile change data {}", message.id);

            // let mut current_time = 0;
            // let result = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
            // if let Ok(elapsed) = result {
            //     current_time = elapsed.as_secs();
            // }


            sequence_number = sequence_number + 1;

            let mut data = tiles_agregator_lock.lock().await;
            
            let old = data.get(&message.id);
            match old {
                Some(_previous_record) => {
                    data.insert(message.id.clone(), message);
                }
                _ => {
                    data.insert(message.id.clone(), message);
                }
            }
        }
    });

    // task that will perdiodically send dta to all clients
    tokio::spawn(async move {
        let mut players_summary = Vec::new();
        loop {
            // assuming 30 fps.
            tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;

            let mut data = processor_lock.lock().await;
            let mut tiles_data = tiles_processor_lock.lock().await;
            if data.len() <= 0  && tiles_data.len() <= 0{
                continue;
            }


            for item in data.iter()
            {
                let cloned_data = item.1.to_owned();
                players_summary.push(cloned_data);
            }


            let mut tiles_summary = Vec::new();
            // since I am clearing the hashmap, maybe there is a way to extract the value to avoid
            for tile in tiles_data.iter()
            {
                tiles_summary.push(tile.1.clone());
            }
            tiles_data.clear();
            let tiles_state_update = tiles_summary.into_iter().map(|t| StateUpdate::TileState(t));

            // we should easily get this lock, since only new clients would trigger a lock on the other side.
            let mut clients_data = players.lock().await;

            // Sending summary to all clients.

            let mut filtered_summary = players_summary.iter()
            // .filter(|p| {
            //     p.sequence_number > client.1.sequence_number
            // })
            .map(|p| StateUpdate::PlayerState(p.clone()))
            .collect::<Vec<StateUpdate>>();

            filtered_summary.extend(tiles_state_update.clone());

            // the data that will be sent to each client is not copied.
            let arc_summary = Arc::new(filtered_summary);

            for client in clients_data.iter_mut()
            {
                if arc_summary.len() > 0
                {
                    // here we send data to the client
                    client.1.tx.send(arc_summary.clone()).await.unwrap();
                }
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