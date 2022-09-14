

use tokio::net::UdpSocket;


const PING: u8 = 1;
const POSITION: u8 = 2;

pub async fn route_packet(socket: &UdpSocket, data : &[u8; 508]){
    match data.get(0) {
        Some(protocol) if *protocol == PING => {
            // ping
            // let num = u16::from_le_bytes(data[1..=2].try_into().unwrap());
            // println!("the message is an {num}");
            let len = socket.send(data).await.unwrap();
            println!("{:?} bytes sent", len);
        },
        Some(protocol) if *protocol == POSITION => {
            // ping
            // let num = u16::from_le_bytes(data[1..=2].try_into().unwrap());
            // prinln!("the message is an {num}");
        },
        _ => {}
    }

}