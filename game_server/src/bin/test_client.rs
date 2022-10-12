use std::sync::Arc;

use game_server::player::player_action::{PlayerAction};
use tokio::{net::{UdpSocket, TcpSocket, TcpStream}, io::{AsyncWriteExt, AsyncReadExt}};
use glam::{Vec3, vec3};
use rand::{rngs::StdRng, Rng};


#[tokio::main]
async fn main() {
    let mut random_generator = <StdRng as rand::SeedableRng>::from_entropy();
    for i in 0..200
    {
        // let x =  random_generator.gen::<f32>();
        // tokio::time::sleep(tokio::time::Duration::from_millis(3000 * (x as u64))).await;
        spawn_test_client(i as u64).await;
    }
    // spawn_test_client(2).await;

    loop{
        tokio::time::sleep(tokio::time::Duration::from_millis(30000)).await;
    }
}

async fn spawn_test_client(client_id : u64) {

    let remote_addr: std::net::SocketAddr = "18.217.145.9:11004".parse().unwrap();
    // let remote_addr: std::net::SocketAddr = "127.0.0.1:11004".parse().unwrap();

    let local_addr: std::net::SocketAddr = if remote_addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }
        .parse().unwrap();

    // let stream = TcpStream::connect("18.217.145.9:11004").await.unwrap();
    let stream = TcpStream::connect("127.0.0.1:11004").await.unwrap();
    println!("created stream");

    let (mut read, mut write) = tokio::io::split(stream);

    // let result = stream.write(b"hello world\n").await;
    // println!("wrote to stream; success={:?}", result.is_ok());

    // let socket = TcpSocket::bind(local_addr).await.unwrap();
    // // const MAX_DATAGRAM_SIZE: usize = 65_507;
    // socket.connect(&remote_addr).await.unwrap();

    // let shareable_socket = Arc::new(socket);

    // let send_socket = shareable_socket.clone();
    // let rec_socket = shareable_socket.clone();
    //send
    tokio::spawn(async move {
        let mut position = Vec3::new(-15.52, 33.74, -297.19);
        let radius = 300f32;

        

        let mut random_generator = <StdRng as rand::SeedableRng>::from_entropy();
        //-15.52, 33.74, -297.19) 299.5 target (-14.51, 35.16, -297.07)

        loop {
            let x =  random_generator.gen::<f32>();
            let y =  random_generator.gen::<f32>();
            let z =  random_generator.gen::<f32>();

            let direction = vec3(x, y, z).normalize();

            let second_position = position + direction * 1f32;
            let second_position = second_position.normalize() * radius;

            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

            // println!("send data {} " ,position);

            let client_action = PlayerAction { 
                player_id:client_id,
                position:[position.x, position.y, position.z],
                second_position:[second_position.x, second_position.y, second_position.z],
                action:1
            };

            position = second_position;

            let bytes = client_action.to_bytes();
            let mut buffer = [0u8; 37];
            buffer[0] = 2;
            buffer[1..37].copy_from_slice(&bytes);

            write.write(&buffer).await.unwrap();

            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

            let client_action = PlayerAction { 
                player_id:client_id,
                position:[position.x, position.y, position.z],
                second_position:[second_position.x, second_position.y, second_position.z],
                action:0
            };

            let bytes = client_action.to_bytes();
            let mut buffer = [0u8; 37];
            buffer[0] = 2;
            buffer[1..37].copy_from_slice(&bytes);

            write.write(&buffer).await.unwrap();
        }
    });


    //receive
    tokio::spawn(async move {
        loop {
            let mut data = vec![0u8; 508];
            let _len = read.read_exact(&mut data).await.unwrap();
            // println!("got some data from server {}", len);
        }
    });


}