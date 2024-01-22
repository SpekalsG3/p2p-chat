use std::io::Write;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

fn handle_connection (mut stream: TcpStream, addr: SocketAddr) {
    println!("---New request from {}", addr);

    loop {
        std::thread::sleep(Duration::from_secs(1));
        match stream.write("hello".as_bytes()) {
            Ok(_) => {
            }
            Err(_) => {
                break;
            }
        };
    };
}

pub fn start_server(server_addr: SocketAddr) {
    let server = TcpListener::bind(server_addr).expect("---Failed to assign udp socket");
    println!("---Listening on {}", server_addr);

    loop {
        match server.accept() {
            Ok((stream, addr)) => {
                handle_connection(stream, addr);
            },
            Err(e) => {
                eprintln!("---Failed to establish connection {}", e);
            }
        }
    }
}
