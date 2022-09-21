use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::player_action::ClientAction;

pub async fn process_ping(socket:&UdpSocket, data : &[u8; 508], _channel_tx : &Sender<ClientAction>)
{
    // ping
    let num = u16::from_le_bytes(data[1..=2].try_into().unwrap());
    // println!("the message is an {num}");
    let len = socket.send(data).await.unwrap();
    // println!("{:?} bytes sent", len);

    // let user1 = ClientAction {
    //     email: String::from("someone@example.com"),
    //     username: String::from("someusername123"),
    //     active: true,
    //     sign_in_count: 1,
    // };
    // channel_tx.send(user1).await.unwrap();
}