
// mod create:utils;

use std::sync::Arc;
use tokio::time;
use tokio::time::Duration;
use tokio::sync::mpsc;

use crate::map::map_entity::{MapEntity, MapCommand};
use crate::player::player_action::PlayerAction;
use crate::player::player_state::PlayerState;
use crate::{protocols, player};


#[derive(Debug)]
pub enum StateUpdate {
    PlayerState(PlayerState),
    TileState(MapEntity),
}

pub async fn spawn_client_process(address : std::net::SocketAddr, 
    from_address : std::net::SocketAddr, 
    channel_tx : mpsc::Sender<std::net::SocketAddr>,
    mut channel_rx : mpsc::Receiver<Arc<Vec<[u8;508]>>>,
    channel_map_action_tx : mpsc::Sender<MapCommand>,
    channel_action_tx : mpsc::Sender<PlayerAction>,
    initial_data : [u8; 508])
{
    let (kill_tx, mut kill_rx) = mpsc::channel::<u8>(2);

    let child_socket : tokio::net::UdpSocket = super::utils::create_reusable_udp_socket(address);
    child_socket.connect(from_address).await.unwrap();

    let shareable_socket = Arc::new(child_socket);
    let socket_global_send_instance = shareable_socket.clone();
    let socket_local_instance = shareable_socket.clone();

    let mut max_seq = 0;

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
                    for packet in data.iter()
                    {
                        let len = socket_global_send_instance.send(packet).await;
                    }

                }
            }
        }
    });

    //messages from the client to the server, like an updated position
    tokio::spawn(async move {

        //handle the first package
        // I think the first package doesn't matter.
        // packet_router::route_packet(&socket_local_instance, &initial_data, &channel_action_tx).await;

        let mut child_buff = [0u8; 508];
        'main_loop : loop {
            let socket_receive = socket_local_instance.recv(&mut child_buff);
            let time_out = time::sleep(Duration::from_secs_f32(5.0)); 
            tokio::select! {
                result = socket_receive => {

                    match result{
                        Ok(_size) => {
                            // println!("Child: {:?} bytes received on child process for {}", size, from_address);
                            protocols::route_packet(&socket_local_instance, &child_buff, &channel_action_tx, &channel_map_action_tx).await;
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
