use std::{sync::Arc, borrow::Borrow, iter::OnceWith, f32::MAX};

use crate::{player_state::PlayerState, packet_router, player_action::ClientAction};
use tokio::sync::Mutex;
use std::collections::HashMap;



pub fn process_player_action(mut receiver : tokio::sync::mpsc::Receiver<ClientAction>, players : tokio::sync::watch::Sender<Vec<PlayerState>>)
{
    let mut all_players = HashMap::<u64,PlayerState>::new();
    let mut data_mutex = Arc::new(Mutex::new(all_players));

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
                direction : message.direction,
                action : message.action
            };

            // println!("player {} pos {:?}",seq, message.position);
            seq = seq + 1;

            let old = data.get(&message.player_id);
            match old {
                Some(previous_record) => {
                    data.insert(message.player_id, new_client_state);
                }
                _ => {
                    data.insert(message.player_id, new_client_state);
                }
            }
        }
    });


    tokio::spawn(async move {
        let mut buffer = [0u8; 508];
        let mut players_summary = Vec::new();
        loop {
            // assuming 30 fps.
            tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;

            let mut data = processor_lock.lock().await;
            if data.len() <= 0 {
                continue;
            }
            // println!("sendng global state {}", data.len());

            // buffer[0] = packet_router::GLOBAL_STATE;
            // buffer[1] = data.len() as u8;

            // let size: usize = 36;
            // let mut start: usize = 2;

            // let mut stored_bytes:u32 = 0;
            // let mut stored_states:u8 = 0;

            let mut max_seq = 0;
            // println!("clients {}", data.len() );

            for item in data.iter()
            {
                let cloned_data = item.1.to_owned();
                players_summary.push(cloned_data);
                max_seq = std::cmp::max(max_seq, item.1.borrow().sequence_number);
                // let player_state_bytes = item.1.to_bytes(); // 36 bytes
                // let next = start + size;
                // buffer[start..next].copy_from_slice(&player_state_bytes);
                // start = next;

                // stored_bytes = stored_bytes + 36;
                // stored_states = stored_states + 1;

                // if stored_bytes + 36 > 100
                // {
                //     buffer[1] = stored_states;
                //     players.send(buffer).unwrap();

                //     println!("send intermediate package with {} states ", stored_states);

                //     start = 2;
                //     stored_states = 0;
                //     stored_bytes = 0;
                // }
            }

            // if stored_states > 0
            // {
            //     buffer[1] = stored_states;
            //     players.send(buffer).unwrap();
            //     println!("send final package with {} states ", stored_states);
            // }

            // buffer[1] = stored_states;
            players.send(players_summary.clone()).unwrap();
            players_summary.clear();

            if max_seq > 500
            {
                data.retain(|_, v| v.sequence_number > (max_seq - 500));
            }
        }
    });


}