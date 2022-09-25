
// mod create:utils;

use std::sync::Arc;
use tokio::time;
use tokio::time::Duration;
use tokio::sync::mpsc;

use crate::player_action::PlayerAction;
use crate::player_state::PlayerState;
use crate::{utils, packet_router};


pub async fn spawn_client_process(address : std::net::SocketAddr, 
    from_address : std::net::SocketAddr, 
    channel_tx : mpsc::Sender<std::net::SocketAddr>,
    mut channel_rx : mpsc::Receiver<Vec<PlayerState>>,
    channel_action_tx : mpsc::Sender<PlayerAction>,
    initial_data : [u8; 508])
{
    let (kill_tx, mut kill_rx) = mpsc::channel::<u8>(2);

    let child_socket : tokio::net::UdpSocket = utils::create_reusable_udp_socket(address);
    child_socket.connect(from_address).await.unwrap();

    let shareable_socket = Arc::new(child_socket);
    let socket_global_send_instance = shareable_socket.clone();
    let socket_local_instance = shareable_socket.clone();

    let mut sequence_count = 0;

    // messages from the server to the client, like the global state of the world.
    tokio::spawn(async move {
        // let mut external_rx = channel_rx;
            // let message = receiver.recv().await.unwrap();
        'receive_loop : loop {
            // let external_rx_future = external_rx.changed();
            tokio::select! {
                _ = kill_rx.recv() => {
                    println!("killed read task");
                    break 'receive_loop;
                }
                Some(data) = channel_rx.recv()  =>{
                    // here we have a vec of player state. should we filter before getting it here. Or should we handle the view of the players.
                    // I think we shouldn't receive old data, just new state.
                    // but it is easier to handle it here.

                    let mut buffer = [0u8; 508];
                    buffer[0] = packet_router::GLOBAL_STATE;
                    buffer[1] = data.len() as u8;

                    let size: usize = 36;
                    let mut start: usize = 2;

                    let mut stored_bytes:u32 = 0;
                    let mut stored_states:u8 = 0;

                    for player_state in data
                    {
                        let player_state_bytes = player_state.to_bytes();
                        let next = start + size;
                        buffer[start..next].copy_from_slice(&player_state_bytes);
                        start = next;

                        stored_bytes = stored_bytes + 36;
                        stored_states = stored_states + 1;

                        if stored_bytes + 36 > 500
                        {
                            buffer[1] = stored_states;

                            let len = socket_global_send_instance.send(&buffer.clone()).await;
                            if let Ok(_) = len {

                            }
                            else {
                                println!("send error");
                            }

                            // println!("send intermediate package with {} states ", stored_states);

                            start = 2;
                            stored_states = 0;
                            stored_bytes = 0;
                        }
                    }

                    if stored_states > 0
                    {
                        buffer[1] = stored_states;
                        let len = socket_global_send_instance.send(&buffer).await;
                        if let Ok(_) = len {

                        }
                        else {
                            println!("send error");
                        }

                        // println!("send final package with {} states ", stored_states);
                    }
                }
            }
        }
    });

    //messages from the client to the server, like an updated position
    tokio::spawn(async move {

        //handle the first package
        packet_router::route_packet(&socket_local_instance, &initial_data, &channel_action_tx).await;

        let mut child_buff = [0u8; 508];
        'main_loop : loop {
            let socket_receive = socket_local_instance.recv(&mut child_buff);
            let time_out = time::sleep(Duration::from_secs_f32(5.0)); 
            tokio::select! {
                result = socket_receive => {

                    match result{
                        Ok(_size) => {
                            sequence_count = sequence_count + 1;
                            // println!("Child: {:?} bytes received on child process for {}", size, from_address);
                            packet_router::route_packet(&socket_local_instance, &child_buff, &channel_action_tx).await;
                        }
                        Err(error) => {
                            println!("we got an error {:?}", error);
                            break 'main_loop;
                        }
                    }
                }
                _ = time_out => {
                    println!("we couldn't wait any longer sorry!");
                    break 'main_loop;
                }
            }
        }

        // if we are here, this task expired and we need to remove the key from the hashset
        channel_tx.send(from_address).await.unwrap();

        // we also need to kill the send task.
        kill_tx.send(0).await.unwrap();
    });
    // borrowed_socket
}