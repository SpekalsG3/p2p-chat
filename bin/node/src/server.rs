use std::io::Write;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;
use crate::types::state::AppState;

fn handle_connection(
    app_state: AppState,
    mut stream: TcpStream,
    addr: SocketAddr,
) {
    app_state.ui().system_message(&format!("---New request from {}", addr));

    app_state.add_stream(
        addr,
        stream.try_clone().expect("---Failed to clone tcp stream"),
    ).expect("---Failed to save stream");

    let pad = (1..5).into_iter();
    let mut pad_temp = pad.clone();

    loop {
        std::thread::sleep(Duration::from_secs(1));
        let add = match pad_temp.next() {
            Some(n) => "_".repeat(n),
            None => {
                pad_temp = pad.clone();
                let n = pad_temp.next().unwrap();
                "_".repeat(n)
            }
        };
        match stream.write(format!("hello{}", add).as_bytes()) {
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
    app_state.ui().system_message(&format!("---Listening on {}", server_addr));

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
                app_state.ui().system_message(&format!("---Failed to establish connection {}", e));
            }
        }
    }
}
