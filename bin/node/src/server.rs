use std::io::Write;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::time::Duration;
use crate::types::Message;

fn handle_connection (
    tx: Sender<Message>,
    mut stream: TcpStream,
    addr: SocketAddr,
) {
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

pub fn start_server(tx: Sender<Message>, server_addr: SocketAddr) {
    let server = TcpListener::bind(server_addr).expect("---Failed to assign udp socket");
    println!("---Listening on {}", server_addr);

    loop {
        match server.accept() {
            Ok((stream, addr)) => {
                handle_connection(tx.clone(), stream, addr);
            },
            Err(e) => {
                eprintln!("---Failed to establish connection {}", e);
            }
        }
    }
}
