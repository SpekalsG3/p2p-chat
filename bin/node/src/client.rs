use std::net::{SocketAddr, TcpStream};
use crate::protocol::read_stream::protocol_read_stream;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};
use crate::types::state::AppState;

pub fn start_client(
    app_state: AppState,
    client_addr: SocketAddr,
) {
    let client = TcpStream::connect(client_addr).expect("---Failed to connect");
    app_state
        .send_package(AppPackage::Alert(AlertPackage {
            level: AlertPackageLevel::INFO,
            msg: format!("You joined to {}", client_addr),
        }))
        .expect("---Failed to send package");

    app_state.add_stream(
        client_addr,
        client.try_clone().expect("---Failed to clone tcp stream"),
    ).expect("---Failed to save stream to state");

    // todo: it's hardcode, provide choice to the user to change rooms
    app_state.set_selected_room(Some(client_addr))
        .expect("---Failed to set_selected_room");

    protocol_read_stream(
        &app_state,
        client_addr,
        client,
    );
}
