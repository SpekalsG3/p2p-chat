use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use anyhow::{bail, Context, Result};

pub const PROT_OPCODE_CONTINUATION: u8 = 0b0000; // received frame is an continuation of previous unfinished frame
pub const PROT_OPCODE_CONN_CLOSED:  u8 = 0b0001; // party disconnected // todo: send in case of graceful shutdown
pub const PROT_OPCODE_PING:         u8 = 0b0010; // checking if connection is still alive
pub const PROT_OPCODE_PONG:         u8 = 0b0011; // answer if connection is still alive
pub const PROT_OPCODE_DATA:         u8 = 0b0100; // frame contains application data
pub const PROT_OPCODE_NODE_INFO:    u8 = 0b0101; // information about other nodes client chooses to connect/disconnect/etc.

pub const PROTOCOL_BUF_SIZE: usize = 256;

pub enum ProtocolBufferType {
    Data,
    NodeInfo,
    Pong,
}
pub enum ProtocolAction {
    None,
    UpdateBufferType(ProtocolBufferType),
    UseBuffer,
    CloseConnection,
    ReceivedPing,
}

pub struct NodeInfo {
    pub addr: SocketAddr,
    pub ping: u16,
}

impl<'a> NodeInfo {
    pub fn new (addr: SocketAddr, ping: u16) -> Self {
        Self {
            addr,
            ping,
        }
    }

    pub const BYTES: usize = 8;

    pub fn into_bytes(self) -> Result<Vec<u8>> {
        let ip = match self.addr.ip() {
            IpAddr::V4(ip) => ip,
            IpAddr::V6(_) => {
                bail!("Dont support IPv6");
            }
        };
        let port = self.addr.port();

        let mut v = Vec::with_capacity(Self::BYTES);
        v.extend(ip.octets());

        let port_bytes = port.to_be_bytes();
        if port_bytes.len() != 2 { // prevent unexpected update break
            bail!("unexpected length of bytes array")
        }
        v.extend(port_bytes);

        let ping_bytes = self.ping.to_be_bytes();
        if ping_bytes.len() != 2 { // prevent unexpected update break
            bail!("unexpected length of bytes array")
        }
        v.extend(ping_bytes);

        Ok(v)
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let mut iter = bytes.into_iter();

        let ip = Ipv4Addr::new(
            iter.next().context("not enough bytes")?,
            iter.next().context("not enough bytes")?,
            iter.next().context("not enough bytes")?,
            iter.next().context("not enough bytes")?,
        );
        let port = u16::from_be_bytes([
            iter.next().context("not enough bytes")?,
            iter.next().context("not enough bytes")?,
        ]);
        let addr = SocketAddr::new(IpAddr::V4(ip), port);

        let ping = u16::from_be_bytes([
            iter.next().context("not enough bytes")?,
            iter.next().context("not enough bytes")?,
        ]);

        Ok(Self {
            addr,
            ping,
        })
    }
}
