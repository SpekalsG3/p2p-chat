use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread::JoinHandle;
use crate::protocol::encode_frame_data::protocol_encode_frame_data;
use crate::protocol::start_pinging::start_pinging;
use crate::protocol::read_stream::protocol_read_stream;
use crate::protocol::vars::{PROT_OPCODE_UPD_TOPOLOGY, TopologyUpdate};
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};
use crate::types::state::AppState;

fn handle_connection(
    app_state: &AppState,
    addr: SocketAddr,
    mut stream: TcpStream,
) -> [JoinHandle<()>; 2] {
    let mut lock = app_state.write_lock().expect("---Failed to get write lock");

    AppState::send_package(&mut lock, AppPackage::Alert(AlertPackage {
        level: AlertPackageLevel::INFO,
        msg: format!("New join from {}", addr),
    })).expect("---Failed to send app message");

    AppState::add_stream(
        &mut lock,
        addr,
        stream.try_clone().expect("---Failed to clone tcp stream"),
    );

    // todo: it's hardcode, provide choice to the user to change rooms
    AppState::set_selected_room(&mut lock, Some(addr));

    {
        let another_conn = lock.streams.iter().find(|(k, _)| k.eq(&&addr));

        if let Some((addr, _)) = another_conn {
            let mut v = Vec::with_capacity(7);
            v.extend(TopologyUpdate::Connect(addr.clone()).into_bytes().expect("Failed to convert"));
            let frame = protocol_encode_frame_data(
                PROT_OPCODE_UPD_TOPOLOGY,
                &v,
            );
            frame.send_to_stream(&mut stream).expect("---Failed to send frame");
        }
    }

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
    {
        let mut lock = app_state.write_lock().expect("---Failed to get write lock");
        AppState::send_package(
            &mut lock,
            AppPackage::Alert(AlertPackage {
                level: AlertPackageLevel::INFO,
                msg: format!("Listening on {}", server_addr),
            }),
        )
            .expect("---Failed to send package");
    }

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
                let mut lock = app_state.write_lock().expect("---Failed to get write lock");
                AppState::send_package(
                    &mut lock,
                    AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::ERROR,
                        msg: format!("Failed to accept connection - {}", e),
                    }),
                )
                    .expect("---Failed to send package");
            }
        }
    }
}
