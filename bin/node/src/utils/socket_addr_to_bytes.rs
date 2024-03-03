use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::vec::IntoIter;
use anyhow::{bail, Context, Result};

pub fn socket_addr_to_bytes(addr: SocketAddr) -> Result<[u8; 6]> {
    let ip = match addr.ip() {
        IpAddr::V4(ip) => ip,
        IpAddr::V6(_) => {
            bail!("Dont support IPv6");
        }
    };
    let port = addr.port();

    let port_bytes = port.to_be_bytes();
    if port_bytes.len() != 2 { // prevent unexpected update break
        bail!("unexpected length of bytes array")
    }

    let octs = ip.octets();
    let v = [
        octs[0],
        octs[1],
        octs[2],
        octs[3],
        port_bytes[0],
        port_bytes[1],
    ];

    Ok(v)
}

pub fn socket_addr_from_bytes(bytes: &mut IntoIter<u8>) -> Result<Option<SocketAddr>> {
    let ip = Ipv4Addr::new(
        bytes.next().context("not enough bytes")?,
        bytes.next().context("not enough bytes")?,
        bytes.next().context("not enough bytes")?,
        bytes.next().context("not enough bytes")?,
    );
    if ip == Ipv4Addr::new(0,0,0,0) {
        return Ok(None);
    }
    let port = u16::from_be_bytes([
        bytes.next().context("not enough bytes")?,
        bytes.next().context("not enough bytes")?,
    ]);

    Ok(Some(SocketAddr::new(IpAddr::V4(ip), port)))
}
