use std::{sync::Arc, borrow::Borrow};

use crate::{player_state::PlayerState, player_action::PlayerAction, player_entity::PlayerEntity};
use tokio::sync::Mutex;
use std::collections::HashMap;



pub fn process_player_action(
    mut receiver : tokio::sync::mpsc::Receiver<PlayerAction>,
    players : Arc<Mutex<HashMap<std::net::SocketAddr,PlayerEntity>>>){

    let all_players = HashMap::<u64,PlayerState>::new();
    let data_mutex = Arc::new(Mutex::new(all_players));

    let processor_lock = data_mutex.clone();
    let agregator_lock = data_mutex.clone();

    let mut seq = 0;

    //task that will handle receiving and updating the state.
    tokio::spawn(async move {

        let mut sequence_number:u64 = 101;
        loop {

            let message = receiver.recv().await.unwrap();
            // println!("player action received {:?}", message);

            sequence_number = sequence_number + 1;
            // I think this should be the entire state of the client, is it moving ? is it choppoing wood, is it attacking?, etc.
            // and then I just store the current state and if it doesn't change, no problem. If it changes(as it should) we update the state and send again.
            // we can't assume someone receive the message, so we send everything continously.
            
            // we could filter what the client gets based on some version number that the client sends to tell us how up to date it is.
            // we should create one batch of data for everyone, but probably we can make a group of packages based on how old the data is.
            // and then we send the packages based on that version number.
            //packages will have ranges of version.
            // each time we get an action we update the number version++ and give it to the stored data.
            // this could be a simple queue that we can crunch. And we should delete old data if new data is available.
            // so it should be a dictionary.
            // we should try to send new data first and we go back in time removing or ignoring old data. while we recreate a new queue with cleaned data.
            // all that process when making the consolidation.
            
            // but since we only send the consolidated state to each client, each client has to filter before sending.

            let mut data = agregator_lock.lock().await;
            
            // here we have access to the players data;

            let new_client_state = PlayerState{
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


    tokio::spawn(async move {
        // let buffer = [0u8; 508];
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

            if max_seq > 500
            {
                data.retain(|_, v| v.sequence_number > (max_seq - 500));
            }
        }
    });


}