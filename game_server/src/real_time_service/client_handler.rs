
// mod create:utils;

use std::sync::Arc;
use tokio::time;
use tokio::time::Duration;
use tokio::sync::{mpsc};

use crate::map::map_entity::{MapEntity, MapCommand};
use crate::player::player_attack::PlayerAttack;
use crate::player::player_command::PlayerCommand;
use crate::player::player_entity::PlayerEntity;
use crate::player::player_presentation::PlayerPresentation;
use crate::{protocols, ServerState};


#[derive(Debug)]
pub enum StateUpdate {
    PlayerState(PlayerEntity),
    PlayerGreetings(PlayerPresentation), // we should actually send more data...
    TileState(MapEntity),
    PlayerAttackState(PlayerAttack),
}

pub async fn spawn_client_process(
    _player_id : u64,
    address : std::net::SocketAddr, 
    from_address : std::net::SocketAddr, 
    server_state: Arc<ServerState>,
    channel_tx : mpsc::Sender<std::net::SocketAddr>,
    channel_map_action_tx : mpsc::Sender<MapCommand>,
    channel_action_tx : mpsc::Sender<PlayerCommand>,
    _initial_data : [u8; 508])
{
    let child_socket : tokio::net::UdpSocket = super::utils::create_reusable_udp_socket(address);
    child_socket.connect(from_address).await.unwrap();

    let shareable_socket = Arc::new(child_socket);
    let socket_local_instance = shareable_socket.clone();

    //messages from the client to the server, like an updated position
    tokio::spawn(async move {


        // we should try to get the player data at this point!

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
                            protocols::route_packet(
                                &socket_local_instance, 
                                &child_buff, 
                                &server_state,
                                &channel_action_tx, 
                                &channel_map_action_tx).await;
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

    });
    // borrowed_socket
}
