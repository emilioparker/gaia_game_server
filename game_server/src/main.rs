use socket2::Socket;
use socket2::Type;
use tokio::time;
use tokio::time::sleep;
use tokio::time::Duration;
use std::collections::HashSet;
use std::convert::TryInto;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread::yield_now;

// #[tokio::main(worker_threads = 1)]
#[tokio::main]
async fn main() {
    
    let (tx, mut rx ) = tokio::sync::mpsc::channel::<std::net::SocketAddr>(100);
    let mut clients:HashSet<std::net::SocketAddr> = HashSet::new();

    let address: std::net::SocketAddr = "0.0.0.0:11004".parse().unwrap();
    // let address: std::net::SocketAddr = "127.0.0.1:11004".parse().unwrap();
    let udp_socket = create_reusable_udp_socket(address);

    let mut buf_udp = [0u8; 1024];
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

                        let tx = tx.clone();
                        spawn_client_process(address, from_address, tx);
                    }
                }
            }
            Some(res) = rx.recv() => {
                println!("removing entry from hash set");
                clients.remove(&res);
            }
            
        }
    }   
}

fn spawn_client_process(address : std::net::SocketAddr, from_address : std::net::SocketAddr, channel_tx : tokio::sync::mpsc::Sender<std::net::SocketAddr>)
{
    tokio::spawn(async move {
        let child_socket : tokio::net::UdpSocket = create_reusable_udp_socket(address);
        child_socket.connect(from_address).await.unwrap();
        let mut child_buff = [0u8; 1024];
        'main_loop : loop {
            let socket_receive = child_socket.recv(&mut child_buff);
            let time_out = time::sleep(Duration::from_secs_f32(10.0)); 
            tokio::select! {
                result = socket_receive => {
                    println!("we did it");
                    if let Ok(size) = result {
                        println!("Child: {:?} bytes received on child process for {}", size, from_address);
                        let len = child_socket.send(&child_buff[..size]).await.unwrap();
                        println!("{:?} bytes sent", len);
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
}


fn create_reusable_udp_socket(address :std::net::SocketAddr) -> tokio::net::UdpSocket
{
    let socket = Socket::new(socket2::Domain::IPV4, Type::DGRAM, None).unwrap();
    socket.set_reuse_port(true).unwrap();
    socket.set_reuse_address(true).unwrap();
    socket.bind(&address.into()).unwrap();
    socket.set_nonblocking(true).unwrap();
    let udp_socket: std::net::UdpSocket = socket.into();
    udp_socket.try_into().unwrap()
}






        // if let Some(message_type) = buf.get(0)
        // {
        //     println!("got a message of type at 0 {message_type}");
        // }
        // // if let Some(message_type) = buf.get(1)
        // // {
        // //     println!("got a message of type at 1 {message_type}");
        // // }

        // // let content_type_bytes = [buf[1], buf[2]];

        // let num = u16::from_le_bytes(buf[1..=2].try_into().unwrap());
        // println!("the message is an {num}");


        // let len = sock.send_to(&buf[..len], addr).await.unwrap();
        // println!("{:?} bytes sent", len);















async fn spanw_test() {
    println!("Hello, world!");
    // my_function().await;

    let mut handles = vec![];
    handles.push(tokio::spawn(async {
        expensive_computation();
    }));

    for i in 0..2
    {
        let handle = tokio :: spawn(async move {
            my_function(i).await;
        });
        handles.push(handle);
    }

    handles.push(tokio::spawn(async {
        let _res = tokio::task::spawn_blocking(|| {
            expensive_computation();
        }).await;
    }));

    for handle in handles
    {
        handle.await.unwrap();
    }
}

async fn my_function(i : i32)
{
    println!("I am an async function");
    let s1 = read_from_database().await;
    println!("{i} First result  {s1}");

    let s2 = read_from_database().await;
    println!(" {i} Second result: {s2}");

}

async fn read_from_database() -> String
{
    sleep(Duration::from_millis(50)).await;
    "db_result".to_owned()
}

fn expensive_computation()
{
    let mut i : i32 = 0;
    for _ in 0..400_000_000
    {
        i = i + 1;
    }
    println!("Dont with expensive computation! i = {i}");

}
