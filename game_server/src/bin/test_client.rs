use std::sync::Arc;

use game_server::player_action::PlayerAction;
use tokio::net::UdpSocket;


#[tokio::main]
async fn main() {
    

    for i in 0..100
    {
        spawn_test_client(i as u64).await;
    }
    // spawn_test_client(2).await;

    loop{
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
    }
}

async fn spawn_test_client(client_id : u64) {

    // let remote_addr: std::net::SocketAddr = "3.141.30.82:11004".parse().unwrap();
    let remote_addr: std::net::SocketAddr = "127.0.0.1:11004".parse().unwrap();

    let local_addr: std::net::SocketAddr = if remote_addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }
        .parse().unwrap();


    let socket = UdpSocket::bind(local_addr).await.unwrap();
    // const MAX_DATAGRAM_SIZE: usize = 65_507;
    socket.connect(&remote_addr).await.unwrap();

    let shareable_socket = Arc::new(socket);

    let send_socket = shareable_socket.clone();
    let rec_socket = shareable_socket.clone();
    //send
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
            let client_action = PlayerAction{
                player_id:client_id,
                position:[10.1,1.3,45.0],
                direction:[10.1,1.3,45.0],
                action:2
            };
            let bytes = client_action.to_bytes();
            let mut buffer = [0u8; 37];
            buffer[0] = 2;
            buffer[1..37].copy_from_slice(&bytes);

            send_socket.send(&buffer).await.unwrap();
        }
    });


    //receive
    tokio::spawn(async move {
        loop {
            let mut data = vec![0u8; 508];
            let _len = rec_socket.recv(&mut data).await.unwrap();
            // println!("got some data from server {}", len);
        }
    });


}