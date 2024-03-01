use std::net::{SocketAddr, TcpStream};
use crate::protocol::read_stream::protocol_read_stream;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};
use crate::types::state::AppState;

pub fn start_client(
    app_state: AppState,
    client_addr: SocketAddr,
) {
    let client = TcpStream::connect(client_addr).expect("---Failed to connect");

    {
        let mut lock = app_state.write_lock().expect("---Failed to get write lock");

        AppState::send_package(&mut lock, AppPackage::Alert(AlertPackage {
            level: AlertPackageLevel::INFO,
            msg: format!("You joined to {}", client_addr),
        })).expect("---Failed to send app message");

        AppState::add_stream(
            &mut lock,
            client_addr,
            client.try_clone().expect("---Failed to clone tcp stream"),
        );

        // todo: it's hardcode, provide choice to the user to change rooms
        AppState::set_selected_room(&mut lock, Some(client_addr));
    }

    protocol_read_stream(
        app_state,
        client_addr,
        client,
    );
}
