use bytes::Bytes;


#[derive(Debug, Clone)]
pub struct GenericCommand
{
    pub player_address : std::net::SocketAddr, 
    pub data : Bytes
}