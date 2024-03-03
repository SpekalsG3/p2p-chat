use std::net::{IpAddr, SocketAddr};
use anyhow::{bail, Result};

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
