use std::io::Write;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;
use crate::types::state::AppState;

fn handle_connection(
    app_state: AppState,
    mut stream: TcpStream,
    addr: SocketAddr,
) {
    println!("---New request from {}", addr);

    app_state.add_stream(
        addr,
        stream.try_clone().expect("---Failed to clone tcp stream"),
    ).expect("---Failed to save stream");

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

pub fn start_server(
    app_state: AppState,
    server_addr: SocketAddr,
) {
    let server = TcpListener::bind(server_addr).expect("---Failed to assign udp socket");
    println!("---Listening on {}", server_addr);


    loop {
        match server.accept() {
            Ok((stream, addr)) => {
                handle_connection(
                    app_state.clone(),
                    stream,
                    addr,
                );
            },
            Err(e) => {
                eprintln!("---Failed to establish connection {}", e);
            }
        }
    }
}
