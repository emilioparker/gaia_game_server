use socket2::{Socket, Type};


pub fn create_reusable_udp_socket(address :std::net::SocketAddr) -> tokio::net::TcpStream
{
    let socket = Socket::new(socket2::Domain::IPV4, Type::STREAM, None).unwrap();
    socket.set_reuse_port(true).unwrap();
    socket.set_reuse_address(true).unwrap();
    socket.bind(&address.into()).unwrap();
    socket.set_nonblocking(true).unwrap();
    let udp_socket: std::net::TcpStream = socket.into();
    udp_socket.try_into().unwrap()
}