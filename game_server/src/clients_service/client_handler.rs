
// mod create:utils;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio::time;
use tokio::time::Duration;
use tokio::sync::{mpsc};

use crate::ability_user::attack::Attack;
use crate::ability_user::attack_result::AttackResult;
use crate::gameplay_service::generic_command::GenericCommand;
use crate::mob::mob_command::MobCommand;
use crate::mob::mob_instance::MobEntity;
use crate::character::character_command::{CharacterCommand, CharacterMovement};
use crate::character::character_entity::CharacterEntity;
use crate::character::character_presentation::CharacterPresentation;
use crate::character::character_reward::CharacterReward;
use crate::chat::ChatCommand;
use crate::chat::chat_entry::ChatEntry;
use crate::map::GameMap;
use crate::map::map_entity::{MapEntity, MapCommand};
use crate::protocols::disconnect_protocol;
use crate::tower::TowerCommand;
use crate::tower::tower_entity::TowerEntity;
use crate::{protocols, ServerState};


#[derive(Debug)]
pub enum StateUpdate 
{
    PlayerState(CharacterEntity),
    PlayerGreetings(CharacterPresentation), 
    Rewards(CharacterReward),
    TileState(MapEntity),
    TowerState(TowerEntity),
    AttackState(Attack),
    AttackResultState(AttackResult),
    ChatMessage(ChatEntry),
    MobUpdate(MobEntity),
    ServerStatus([u16;10]),
}

pub async fn spawn_client_process(
    player_id : u16,
    session_id : u64,
    address : std::net::SocketAddr, 
    from_address : std::net::SocketAddr, 
    map : Arc<GameMap>,
    server_state: Arc<ServerState>,
    tx_gc_clients_gameplay : mpsc::Sender<GenericCommand>,
    channel_tx : mpsc::Sender<(std::net::SocketAddr, u64)>,
    tx_mc_clients_gameplay : mpsc::Sender<MapCommand>,
    tx_moc_clients_gameplay : mpsc::Sender<MobCommand>,
    tx_pc_clients_gameplay : mpsc::Sender<CharacterCommand>,
    tx_tc_clients_gameplay : mpsc::Sender<TowerCommand>,
    tx_cc_clients_gameplay : mpsc::Sender<ChatCommand>,
    missing_packets : Arc<HashMap<u16, [AtomicU64;10]>>,
    initial_data : [u8; 508],
    packet_size: usize)
{
    cli_log::info!("------ create reusable socket {} from {}", address, from_address);

    // let socket_sender : tokio::net::UdpSocket = super::utils::create_reusable_udp_socket(address);

    let child_socket : tokio::net::UdpSocket = super::utils::create_reusable_udp_socket(address);
    child_socket.connect(from_address).await.unwrap();
    let socket_receiver = Arc::new(child_socket);


    //messages from the client to the server, like an updated position
    tokio::spawn(async move {
        // we should try to get the player data at this point!

        //handle the first package
        protocols::route_packet(
            player_id,
            from_address,
            &initial_data, 
            packet_size,
            map.clone(),
            &server_state,
            missing_packets.clone(),
            &tx_gc_clients_gameplay,
            &tx_pc_clients_gameplay, 
            &tx_mc_clients_gameplay,
            &tx_moc_clients_gameplay,
            &tx_tc_clients_gameplay,
            &tx_cc_clients_gameplay,
        ).await;

        let mut child_buff = [0u8; 508];
        'main_loop : loop 
        {
            let socket_receive = socket_receiver.recv(&mut child_buff);
            let time_out = time::sleep(Duration::from_secs_f32(10.0)); 
            tokio::select! {
                result = socket_receive => 
                {
                    // read the player id and the session id and drop if session id is different

                    match result
                    {
                        Ok(packet_size) => 
                        {
                            // cli_log::info!("Child: {:?} bytes received on child process for {}", size, from_address);
                            protocols::route_packet(
                                player_id,
                                from_address,
                                &child_buff, 
                                packet_size,
                                map.clone(),
                                &server_state,
                                missing_packets.clone(),
                                &tx_gc_clients_gameplay,
                                &tx_pc_clients_gameplay, 
                                &tx_mc_clients_gameplay,
                                &tx_moc_clients_gameplay,
                                &tx_tc_clients_gameplay,
                                &tx_cc_clients_gameplay,
                            ).await;
                        }
                        Err(error) => 
                        {
                            cli_log::info!("we got an error {:?}", error);
                            break 'main_loop;
                        }
                    }
                }
                _ = time_out => 
                {
                    cli_log::info!("we couldn't wait any longer sorry!");
                    break 'main_loop;
                }
            }
        }

        // before disconnecting, we set action to 0, to indicate that the player is not active
        disconnect_protocol::process(player_id, &tx_pc_clients_gameplay).await;

        // if we are here, this task expired and we need to remove the key from the hashset
        channel_tx.send((from_address, session_id)).await.unwrap();

    });
    // borrowed_socket
}
