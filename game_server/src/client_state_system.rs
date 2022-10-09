use std::mem;
use std::{sync::Arc, borrow::Borrow, time::SystemTime};

use crate::map::map_entity::{MapCommand, MapCommandInfo};
use crate::player::{player_state::PlayerState, player_action::PlayerAction, player_entity::PlayerEntity};
use crate::map::{tetrahedron_id::TetrahedronId, map_entity::MapEntity};
use crate::real_time_service::client_handler::StateUpdate;
use tokio::{sync::Mutex};
use std::collections::HashMap;


pub fn process_player_action(
    mut action_receiver : tokio::sync::mpsc::Receiver<PlayerAction>,
    // mut tile_changed_sender : tokio::sync::mpsc::Sender<MapCommand>,
    mut tile_changed_receiver : tokio::sync::mpsc::Receiver<MapCommand>,
    tiles : Arc<Mutex<HashMap<TetrahedronId, MapEntity>>>,
    players : Arc<Mutex<HashMap<std::net::SocketAddr,PlayerEntity>>>){

    //players
    let all_players = HashMap::<u64,PlayerState>::new();
    let data_mutex = Arc::new(Mutex::new(all_players));
    let processor_lock = data_mutex.clone();
    let agregator_lock = data_mutex.clone();


    //tile commands, this means that many players might hit the same tile, but only one every 30 ms will apply, this is really cool
    //should I add luck to improve the probability of someones actions to hit the tile ?
    let tile_commands = HashMap::<TetrahedronId,MapCommand>::new();
    let tile_commands_mutex = Arc::new(Mutex::new(tile_commands));

    let tile_commands_processor_lock = tile_commands_mutex.clone();
    let tile_commands_agregator_lock = tile_commands_mutex.clone();

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
            
            let new_client_state = PlayerState{
                current_time,
                sequence_number,
                player_id : message.player_id,
                position : message.position,
                second_position : message.direction,
                action : message.action
            };

            // println!("player {} pos {:?}",seq, message.position);
            seq = seq + 1;

            let old = data.get(&message.player_id);
            match old {
                Some(_previous_record) => {
                    data.insert(message.player_id, new_client_state);
                }
                _ => {
                    data.insert(message.player_id, new_client_state);
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

            let mut data = tile_commands_agregator_lock.lock().await;
            
            let old = data.get(&message.id);
            match old {
                Some(_previous_record) => {
                    // this command will be lost for this tile, check if it is important ??
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
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

            let mut data = processor_lock.lock().await;
            let mut tile_commands_data = tile_commands_processor_lock.lock().await;
            if data.len() <= 0  && tile_commands_data.len() <= 0{
                continue;
            }

            for item in data.iter()
            {
                let cloned_data = item.1.to_owned();
                players_summary.push(cloned_data);
            }

            let mut tiles = tiles.lock().await;

            let mut tiles_summary : Vec<MapEntity>= Vec::new();

            for tile_command in tile_commands_data.iter()
            {
                match tiles.get_mut(tile_command.0) {
                    Some(tile) => {
                        let mut updated_tile = tile.clone();
                        // in theory we do something cool here with the tile!!!!
                        match tile_command.1.info {
                            MapCommandInfo::Touch() => {
                                tiles_summary.push(updated_tile);
                            },
                            MapCommandInfo::ChangeHealth(value) => {
                                updated_tile.health = i32::max(0, updated_tile.health as i32 - value as i32) as u32;
                                tiles_summary.push(updated_tile.clone());
                                *tile = updated_tile;
                            }
                        }
                    }
                    _ => (),
                }
            }

            // we want to free the lock as soon as possible.
            // I am not sure if rust automatically releases de lock once there are no more references.
            drop(tiles);

            tile_commands_data.clear();
            data.clear();

            drop(tile_commands_data);
            drop(data);

            let tiles_state_update = tiles_summary.into_iter().map(|t| StateUpdate::TileState(t));

            // we should easily get this lock, since only new clients would trigger a lock on the other side.
            let mut clients_data = players.lock().await;

            // Sending summary to all clients.

            let mut filtered_summary = players_summary.iter()
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
                    if let Ok(_) = client.1.tx.send(arc_summary.clone()).await {

                    }
                    else {
                        println!("Error sending summary to client");
                    }
                }
            }


            players_summary.clear();

            // let result = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
            // if let Ok(elapsed) = result {
            //     let current_time = elapsed.as_secs();
            //     data.retain(|_, v| (current_time - v.current_time) < 5);
            // }
        }
    });
}