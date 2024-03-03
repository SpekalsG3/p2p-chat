use std::net::{SocketAddr, TcpListener};
use std::thread::JoinHandle;
use crate::protocol::read_stream::protocol_read_stream;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};
use crate::types::state::AppState;

fn running_server(
    app_state: AppState,
    server: TcpListener,
) {
    let mut handles = vec![];

    loop {
        match server.accept() {
            Ok((stream, _addr)) => { // I don't think I need an address of local socket
                let h = {
                    let app_state = app_state.clone();
                    std::thread::spawn(move || {
                        protocol_read_stream(
                            app_state,
                            stream,
                        );
                    })
                };
                handles.push(h);
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
