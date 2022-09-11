use socket2::Socket;
use socket2::Type;
use tokio::time::sleep;
use tokio::time::Duration;
use std::collections::HashSet;

#[tokio::main]
async fn main() {
    let mut clients:HashSet<std::net::SocketAddr> = HashSet::new();
    let address: std::net::SocketAddr = "0.0.0.0:11004".parse().unwrap();
    // let address: std::net::SocketAddr = "127.0.0.1:11004".parse().unwrap();
    let udp_socket = create_reusable_udp_socket(address);

    let mut buf_udp = [0u8; 1024];
    loop {
        let result = udp_socket.recv_from(&mut buf_udp);
        if let Ok((size, from_address)) = result {
            println!("Parent: {:?} bytes received from {}", size, from_address);
            if !clients.contains(&from_address)
            {
                println!("--- create child!");
                clients.insert(from_address);
                spawn_client_process(address, from_address);
            }
        }
    }   
}


fn spawn_client_process(address : std::net::SocketAddr, from_address : std::net::SocketAddr)
{
    tokio::spawn(async move {
        let child_socket : std::net::UdpSocket = create_reusable_udp_socket(address);
        println!("create child socket");
        child_socket.connect(from_address).unwrap();
        let mut child_buff = [0u8; 1024];
        loop {
            let result = child_socket.recv(&mut child_buff);
            if let Ok(size) = result {
                println!("Child: {:?} bytes received on child process for {}", size, from_address);
            }
            else {
              println!("error on child socket");
            }
        }
    });
}

fn create_reusable_udp_socket(address :std::net::SocketAddr) -> std::net::UdpSocket
{
    let socket = Socket::new(socket2::Domain::IPV4, Type::DGRAM, None).unwrap();
    socket.set_reuse_port(true).unwrap();
    socket.set_reuse_address(true).unwrap();
    socket.bind(&address.into()).unwrap();
    // socket.set_nonblocking(nonblocking)
    let udp_socket: std::net::UdpSocket = socket.into();
    udp_socket
}






















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
