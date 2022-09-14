use std::collections::HashSet;

mod packet_router;
mod utils;
mod client_handler;

#[tokio::main(worker_threads = 1)]
// #[tokio::main]
async fn main() {
    
    let (from_client_to_world_tx, mut from_client_task_to_parent_rx ) = tokio::sync::mpsc::channel::<std::net::SocketAddr>(100);

    let initial_value = [0u8; 508];
    let (main_data_tx, childs_rx) = tokio::sync::watch::channel(initial_value);

    let mut clients:HashSet<std::net::SocketAddr> = HashSet::new();

    let address: std::net::SocketAddr = "0.0.0.0:11004".parse().unwrap();
    // let address: std::net::SocketAddr = "127.0.0.1:11004".parse().unwrap();
    let udp_socket = utils::create_reusable_udp_socket(address);

    let mut buf_udp = [0u8; 508];
    loop {
        let socket_receive = udp_socket.recv_from(&mut buf_udp);

        tokio::select! {
            result = socket_receive => {
                if let Ok((size, from_address)) = result {
                    println!("Parent: {:?} bytes received from {}", size, from_address);
                    if !clients.contains(&from_address)
                    {
                        println!("--- create child!");
                        clients.insert(from_address);

                        let tx = from_client_to_world_tx.clone();
                        client_handler::spawn_client_process(address, from_address, tx, childs_rx.clone(), buf_udp).await;
                    }
                }
            }
            Some(res) = from_client_task_to_parent_rx.recv() => {
                println!("removing entry from hash set");
                clients.remove(&res);
            }
            
        }
    }   
}


