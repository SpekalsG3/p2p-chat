use std::net::{SocketAddr, TcpListener, TcpStream};
use crate::protocol::encode_frame_data::protocol_encode_frame_data;
use crate::protocol::read_stream::protocol_read_stream;
use crate::protocol::vars::PROT_OPCODE_PING;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};
use crate::types::state::AppState;

fn handle_connection(
    app_state: AppState,
    addr: SocketAddr,
    stream: TcpStream,
) {
    app_state
        .send_package(AppPackage::Alert(AlertPackage {
            level: AlertPackageLevel::INFO,
            msg: format!("New join from {}", addr),
        }))
        .expect("---Failed to send package");

    {
        let frame = protocol_encode_frame_data(PROT_OPCODE_PING, &[]);
        app_state.send_stream_message(&addr, frame).expect("")
    }

    app_state.add_stream(
        addr,
        stream.try_clone().expect("---Failed to clone tcp stream"),
    ).expect("---Failed to save stream");


    // todo: it's hardcode, provide choice to the user to change rooms
    app_state.set_selected_room(Some(addr))
        .expect("---Failed to set_selected_room");

    protocol_read_stream(
        &app_state,
        addr,
        stream,
    );
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
                let app_state = app_state.clone();
                let h = std::thread::spawn(move || {
                    handle_connection(
                        app_state,
                        addr,
                        stream,
                    );
                });
                handles.push(h);
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
