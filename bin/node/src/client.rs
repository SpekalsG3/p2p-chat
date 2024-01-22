use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::Sender;
use crate::types::Message;

pub fn start_client (tx: Sender<Message>, client_addr: SocketAddr) {
    let mut client = TcpStream::connect(client_addr).expect("---Failed to connect");
    println!("---Connected to the server at {}", client_addr);

    loop {
        let mut buf = [0; 256];
        match client.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    continue;
                }
                tx.send(Message {
                    author: client_addr,
                    msg: String::from_utf8_lossy(&buf).to_string(),
                }).expect("---failed to send msg through channel");
            }
            Err(e) => {
                eprintln!("---failed to read {}", e)
            }
        }
    }
}
