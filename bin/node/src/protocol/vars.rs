use std::net::{IpAddr, SocketAddr};
use anyhow::{bail, Result};

pub const PROT_OPCODE_CONTINUATION: u8 = 0b0000; // received frame is an continuation of previous unfinished frame
pub const PROT_OPCODE_CONN_CLOSED:  u8 = 0b0001; // party disconnected // todo: send in case of graceful shutdown
pub const PROT_OPCODE_PING:         u8 = 0b0010; // checking if connection is still alive
pub const PROT_OPCODE_PONG:         u8 = 0b0011; // answer if connection is still alive
pub const PROT_OPCODE_DATA:         u8 = 0b0100; // frame contains application data
pub const PROT_OPCODE_UPD_TOPOLOGY: u8 = 0b0101; // request to connect/reconnect/disconnect to provided IPs

pub const PROTOCOL_BUF_SIZE: usize = 256;

pub enum ProtocolBufferType {
    Data,
    TopologyUpd,
}
pub enum ProtocolAction {
    None,
    UpdateBufferType(ProtocolBufferType),
    UseBuffer,
    CloseConnection,
    Send(Vec<u8>),
    ReceivedPong,
}

pub enum TopologyUpdate {
    Connect(SocketAddr),
}

impl<'a> TopologyUpdate {
    pub fn into_bytes(self) -> Result<Vec<u8>> {
        let bytes = match self {
            TopologyUpdate::Connect(addr) => {
                let ip = match addr.ip() {
                    IpAddr::V4(ip) => ip,
                    IpAddr::V6(_) => {
                        bail!("Dont support IPv6");
                    }
                };
                let port = addr.port();

                let mut v = vec![0];
                v.extend(ip.octets());

                let port_bytes = port.to_be_bytes();
                if port_bytes.len() != 2 {
                    bail!("unexpected length of bytes array")
                }
                v.extend(port_bytes);

                v
            }
        };
        Ok(bytes)
    }
}
