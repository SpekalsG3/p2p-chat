use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread::JoinHandle;
use crate::protocol::ping_pong::start_pinging;
use crate::protocol::read_stream::protocol_read_stream;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};
use crate::types::state::AppState;

fn handle_connection(
    app_state: &AppState,
    addr: SocketAddr,
    stream: TcpStream,
) -> [JoinHandle<()>; 2] {
    app_state
        .send_package(AppPackage::Alert(AlertPackage {
            level: AlertPackageLevel::INFO,
            msg: format!("New join from {}", addr),
        }))
        .expect("---Failed to send package");

    app_state.add_stream(
        addr,
        stream.try_clone().expect("---Failed to clone tcp stream"),
    ).expect("---Failed to save stream");

    // todo: it's hardcode, provide choice to the user to change rooms
    app_state.set_selected_room(Some(addr))
        .expect("---Failed to set_selected_room");

    let ping_handle = {
        let app_state = app_state.clone();
        std::thread::spawn(move || {
            start_pinging(app_state, addr)
        })
    };

    let read_handle = {
        let app_state = app_state.clone();
        std::thread::spawn(move || {
            protocol_read_stream(
                app_state,
                addr,
                stream,
            );
        })
    };

    [ping_handle, read_handle]
}

pub fn start_server(
    app_state: AppState,
    server_addr: SocketAddr,
) {
    let server = TcpListener::bind(server_addr).expect("---Failed to assign udp socket");
    app_state
        .send_package(AppPackage::Alert(AlertPackage {
            level: AlertPackageLevel::INFO,
            msg: format!("Listening on {}", server_addr),
        }))
        .expect("---Failed to send package");

    let mut handles = vec![];

    loop {
        match server.accept() {
            Ok((stream, addr)) => {
                let h = handle_connection(
                    &app_state,
                    addr,
                    stream,
                );
                handles.extend(h);
            },
            Err(e) => {
                app_state
                    .send_package(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::ERROR,
                        msg: format!("Failed to accept connection - {}", e),
                    }))
                    .expect("---Failed to send package");
            }
        }
    }
}
