use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread::JoinHandle;
use crate::protocol::encode_frame_data::protocol_encode_frame_data;
use crate::protocol::start_pinging::start_pinging;
use crate::protocol::read_stream::protocol_read_stream;
use crate::protocol::vars::{NodeInfo, PROT_OPCODE_NODE_INFO};
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};
use crate::types::state::{AppState, MetaData};

fn handle_connection(
    app_state: &AppState,
    addr: SocketAddr,
    mut stream: TcpStream,
) -> [JoinHandle<()>; 2] {
    let mut lock = app_state.write_lock().expect("---Failed to get write lock");

    lock
        .package_sender
        .send(AppPackage::Alert(AlertPackage {
            level: AlertPackageLevel::INFO,
            msg: format!("New join from {}", addr),
        }))
        .expect("---Failed to send app package");

    lock.streams.insert(addr, (
        stream.try_clone().expect("---Failed to clone tcp stream"),
        MetaData {
            ping: 0,
            ping_started_at: None,
            topology_rad: 0_f32,
        },
    ));

    // todo: it's hardcode, provide choice to the user to change rooms
    AppState::set_selected_room(&mut lock, Some(addr));

    {
        let another_conn = lock.streams.iter().find(|(k, _)| k.eq(&&addr));

        if let Some((addr, (_, metadata))) = another_conn {
            let mut v = Vec::with_capacity(NodeInfo::BYTES);
            v.extend(
                NodeInfo::new(addr.clone(), metadata.ping)
                    .into_bytes()
                    .expect("---Failed to convert NodeInfo to bytes")
            );

            let frame = protocol_encode_frame_data(
                PROT_OPCODE_NODE_INFO,
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

fn running_server(
    app_state: AppState,
    server: TcpListener,
) {
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
                let lock = app_state.write_lock().expect("---Failed to get write lock");
                lock
                    .package_sender
                    .send(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::ERROR,
                        msg: format!("Failed to accept connection - {}", e),
                    }))
                    .expect("---Failed to send app package");
            }
        }
    }
}

pub fn start_server(
    app_state: AppState,
    server_addr: SocketAddr,
) -> Option<JoinHandle<()>> {
    let server = {
        let mut lock = app_state.write_lock().expect("---Failed to get write lock");

        if let Some(server_addr) = lock.server_addr {
            lock
                .package_sender
                .send(AppPackage::Alert(AlertPackage {
                    level: AlertPackageLevel::WARNING,
                    msg: format!("Server is already running on {}", server_addr),
                }))
                .expect("---Failed to send app package");

            return None;
        }

        let server = TcpListener::bind(server_addr).expect("---Failed to assign udp socket");

        lock
            .package_sender
            .send(AppPackage::Alert(AlertPackage {
                level: AlertPackageLevel::INFO,
                msg: format!("Listening on {}", server_addr),
            }))
            .expect("---Failed to send app package");

        lock.server_addr = Some(server_addr);

        server
    };

    Some(
        std::thread::spawn(|| {
            running_server(app_state, server)
        })
    )
}
