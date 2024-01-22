use std::net::SocketAddr;

#[derive(Debug)]
pub struct Message {
    pub author: SocketAddr,
    pub msg: String,
}
