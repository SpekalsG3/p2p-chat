use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage, MessagePackage};
use crate::types::state::AppState;

pub fn start_client(
    app_state: AppState,
    client_addr: SocketAddr,
) {
    let mut client = TcpStream::connect(client_addr).expect("---Failed to connect");
    app_state
        .send_package(AppPackage::Alert(AlertPackage {
            level: AlertPackageLevel::INFO,
            msg: format!("Connected to the server at {}", client_addr),
        }))
        .expect("---Failed to send package");

    app_state.add_stream(
        client_addr,
        client.try_clone().expect("---Failed to clone tcp stream"),
    ).expect("---Failed to save stream to state");

    {
        let mut lock = app_state.0.m.write().expect("---Failed to take lock");
        lock.selected_room = Some(client_addr)
    }

    loop {
        let mut buf = [0; 256];
        match client.read(&mut buf) {
            Ok(n) => {
                if n == 0 { // means socket is empty, nothing to read/nothing was sent
                    continue;
                }

                app_state.send_package(AppPackage::Message(MessagePackage {
                    from: client_addr,
                    msg: buf.to_vec(),
                })).expect("---failed to send msg through channel");
            }
            Err(e) => {
                app_state
                    .send_package(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::ERROR,
                        msg: format!("Failed to read stream - {}", e),
                    }))
                    .expect("---Failed to send package");
            }
        }
    }
}
