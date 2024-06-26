use std::net::SocketAddr;
use anyhow::{bail, Context, Result};
use crate::utils::socket_addr_to_bytes::{socket_addr_from_bytes, socket_addr_to_bytes};

#[derive(Debug)]
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
        let mut v = Vec::with_capacity(Self::BYTES);

        v.extend(socket_addr_to_bytes(self.addr)?);

        let ping_bytes = self.ping.to_be_bytes();
        if ping_bytes.len() != 2 { // prevent unexpected update break
            bail!("unexpected length of bytes array")
        }
        v.extend(ping_bytes);

        Ok(v)
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Option<Self>> {
        let mut iter = bytes.into_iter();

        let addr = match socket_addr_from_bytes(&mut iter)? {
            Some(s) => s,
            None => {
                return Ok(None);
            }
        };

        let ping = u16::from_be_bytes([
            iter.next().context("not enough bytes")?,
            iter.next().context("not enough bytes")?,
        ]);

        Ok(Some(Self {
            addr,
            ping,
        }))
    }
}
