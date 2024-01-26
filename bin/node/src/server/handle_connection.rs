use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use crate::types::package::{AppPackage, PackageMessage};
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
    {
        let mut lock = app_state.0.m.write().expect("---Failed to take lock");
        lock.selected_room = Some(addr)
    }

    loop {
        let mut buf = [0; 256];
        match stream.read(&mut buf) {
            Ok(n) => {
                if n == 0 { // means socket is empty, nothing to read/nothing was sent
                    continue;
                }

                app_state.send_package(AppPackage::Message(PackageMessage {
                    from: addr,
                    msg: buf.to_vec(),
                })).expect("---failed to send msg through channel");
            }
            Err(e) => {
                app_state.ui().system_message(&format!("---failed to read {}", e))
            }
        }
    }
}

pub fn start_server(
    app_state: AppState,
    server_addr: SocketAddr,
) {
    let server = TcpListener::bind(server_addr).expect("---Failed to assign udp socket");
    app_state.ui().system_message(&format!("---Listening on {}", server_addr));

    let mut handles = vec![];

    // loop {
        match server.accept() {
            Ok((stream, addr)) => {
                let app_state = app_state.clone();
                let h = std::thread::spawn(move || {
                    handle_connection(
                        app_state,
                        stream,
                        addr,
                    );
                });
                handles.push(h);
            },
            Err(e) => {
                app_state.ui().system_message(&format!("---Failed to establish connection {}", e));
            }
        }
    // }

    for h in handles {
        h.join().expect("---failed to join");
    }
}
