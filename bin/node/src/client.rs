use std::net::{SocketAddr, TcpStream};
use std::thread::JoinHandle;
use crate::protocol::read_stream::protocol_read_stream;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};
use crate::types::state::AppState;

pub fn start_client(
    app_state: AppState,
    addr: SocketAddr,
) -> [JoinHandle<()>; 1] {
    let stream = TcpStream::connect(addr).expect("---Failed to connect");

    {
        let mut lock = app_state.write_lock().expect("---Failed to get write lock");

        AppState::send_package(&mut lock, AppPackage::Alert(AlertPackage {
            level: AlertPackageLevel::INFO,
            msg: format!("You joined to {}", addr),
        })).expect("---Failed to send app message");

        AppState::add_stream(
            &mut lock,
            addr,
            stream.try_clone().expect("---Failed to clone tcp stream"),
        );

        // todo: it's hardcode, provide choice to the user to change rooms
        AppState::set_selected_room(&mut lock, Some(addr));
    }

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

    [read_handle]
}
