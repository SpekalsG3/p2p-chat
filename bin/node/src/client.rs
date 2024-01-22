use std::io::Read;
use std::net::{SocketAddr, TcpStream};

pub fn start_client (client_addr: SocketAddr) {
    let mut client = TcpStream::connect(client_addr).expect("---Failed to connect");
    println!("---Connected to the server at {}", client_addr);

    loop {
        let mut buf = [0; 256];
        match client.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    continue;
                }
                println!("---read {} bytes\n'{}'", n, String::from_utf8_lossy(&buf))
            }
            Err(e) => {
                eprintln!("---failed to read {}", e)
            }
        }
    }
}
