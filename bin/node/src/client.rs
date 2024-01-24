use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use crate::types::package::{AppPackage, PackageMessage};
use crate::types::state::AppState;

pub fn start_client(
    app_state: AppState,
    client_addr: SocketAddr,
) {
    let mut client = TcpStream::connect(client_addr).expect("---Failed to connect");
    println!("---Connected to the server at {}", client_addr);

    app_state.add_stream(
        client_addr,
        client.try_clone().expect("---Failed to clone tcp stream"),
    ).expect("---Failed to save stream to state");

    loop {
        let mut buf = [0; 256];
        match client.read(&mut buf) {
            Ok(n) => {
                if n == 0 { // means socket is empty, nothing to read/nothing was sent
                    continue;
                }

                app_state.send_package(AppPackage::Message(PackageMessage {
                    from: client_addr,
                    msg: buf.to_vec(),
                })).expect("---failed to send msg through channel");
            }
            Err(e) => {
                eprintln!("---failed to read {}", e)
            }
        }
    }
}
