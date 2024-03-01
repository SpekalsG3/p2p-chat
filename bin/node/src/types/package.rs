use std::fmt::{Display, Formatter};
use std::net::SocketAddr;

#[derive(Debug)]
pub enum AppPackage {
    Message(MessagePackage),
    Alert(AlertPackage),
}

#[derive(Debug)]
pub struct MessagePackage {
    pub from: SocketAddr,
    pub msg: Vec<u8>,
}

#[derive(Debug)]
pub enum AlertPackageLevel {
    #[allow(unused)]
    DEBUG,
    ERROR,
    INFO,
    WARNING,
}

impl Display for AlertPackageLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct AlertPackage {
    pub level: AlertPackageLevel,
    pub msg: String,
}
