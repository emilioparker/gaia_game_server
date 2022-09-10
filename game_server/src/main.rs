use tokio::time::sleep;
use tokio::time::Duration;
use tokio::net::UdpSocket;
use std::convert::TryInto;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let sock = UdpSocket::bind("127.0.0.1:11004").await.unwrap();
    let mut buf = [0; 1024];
    loop {
        let (len, addr) = sock.recv_from(&mut buf).await.unwrap();
        println!("{:?} bytes received from {:?}", len, addr);

        if let Some(message_type) = buf.get(0)
        {
            println!("got a message of type at 0 {message_type}");
        }
        // if let Some(message_type) = buf.get(1)
        // {
        //     println!("got a message of type at 1 {message_type}");
        // }

        // let content_type_bytes = [buf[1], buf[2]];

        let num = u16::from_le_bytes(buf[1..=2].try_into().unwrap());
        println!("the message is an {num}");


        let len = sock.send_to(&buf[..len], addr).await.unwrap();
        println!("{:?} bytes sent", len);
    }
}


// #[tokio::main]
// async fn main() {
//     let num_cpus = num_cpus::get();

//     println!("num cpus {num_cpus}");

//     let socketAddress = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8084);

//     let main_socket = new_udp_reuseport(socketAddress);
    // main_socket.connect(addr).await.unwrap();

    // for i in 0..num_cpus {
    //     tokio::spawn(async move {
    //         let mut buf_udp = [0u8; MAX_PACKET_LEN];
    //         let udp_sock = new_udp_reuseport(local_addr);
    //         udp_sock.connect(addr).await.unwrap();
    
    //         loop {
    //             if Ok(size) = udp_sock.recv(&mut buf_udp) {
    //                 println!("{:?} bytes received" size);
    //             }   
    //         }   
    //     }   
    // }
// }

fn new_udp_reuseport(addr: SocketAddr) -> UdpSocket {
    let udp_sock = socket2::Socket::new(
        if addr.is_ipv4() {
            socket2::Domain::IPV4
        } else {
            socket2::Domain::IPV6
        },
        socket2::Type::DGRAM,
        None,
    )
    .unwrap();
    udp_sock.set_reuse_port(true).unwrap();
    // from tokio-rs/mio/blob/master/src/sys/unix/net.rs
    udp_sock.set_cloexec(true).unwrap();
    udp_sock.set_nonblocking(true).unwrap();
    udp_sock.bind(&socket2::SockAddr::from(addr)).unwrap();
    let udp_sock: std::net::UdpSocket = udp_sock.into();
    udp_sock.try_into().unwrap()
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
