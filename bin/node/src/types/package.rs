use std::net::SocketAddr;

#[derive(Debug)]
pub enum AppPackage {
    Message(PackageMessage),
    NewConn(PackageConnData),
}

#[derive(Debug)]
pub struct PackageMessage {
    pub from: SocketAddr,
    pub msg: Vec<u8>,
}

#[derive(Debug)]
pub struct PackageConnData {
    pub addr: SocketAddr,
}
