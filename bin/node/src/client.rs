use std::net::{Shutdown, SocketAddr, TcpStream};
use std::thread::JoinHandle;
use std::time::SystemTime;
use crate::protocol::read_stream::protocol_read_stream;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};
use crate::types::state::{AppState, MetaData};

pub fn start_client(
    app_state: AppState,
    addr: SocketAddr,
    src_info: Option<(SocketAddr, u16)>,
) -> Option<JoinHandle<()>> {
    let ping = SystemTime::now();
    let stream = TcpStream::connect(addr).expect("---Failed to connect");
    let ping = SystemTime::now().duration_since(ping).expect("Failed to calculate ping").as_millis();

    {
        let mut lock = app_state.write_lock().expect("---Failed to get write lock");

        // u16 < 2^24 => save to convert to f32
        // src - https://stackoverflow.com/a/41651053
        if ping > 60_000 { // todo: move to constant
            lock
                .package_sender
                .send(AppPackage::Alert(AlertPackage {
                    level: AlertPackageLevel::WARNING,
                    msg: format!("Ping with host {} is too big ({}). Disconnecting", addr, ping),
                }))
                .expect("---Failed to send app package");
            stream.shutdown(Shutdown::Both).expect("---Failed to shutdown stream");
            return None;
        }
        let ping = ping as u16;

        lock
            .package_sender
            .send(AppPackage::Alert(AlertPackage {
                level: AlertPackageLevel::INFO,
                msg: format!("You joined to {}", addr),
            }))
            .expect("---Failed to send app package");

        let mut stream_metadata = MetaData {
            ping,
            ping_started_at: None,
            topology_rad: 0_f32,
        };
        let ping = stream_metadata.ping;

        // todo: it's hardcode, provide choice to the user to change rooms
        AppState::set_selected_room(&mut lock, Some(addr));

        if let Some((src_addr, src_ping)) = src_info {
            let a = src_ping;
            let b = ping;
            let c = lock.streams.get_mut(&src_addr).expect("src_addr should exist").1.ping;

            let angle = ((c * c + a * a - b * b) as f32 / (2 * c * a) as f32).acos();

            stream_metadata.topology_rad = angle;
        }

        lock.streams.insert(addr, (
            stream.try_clone().expect("---Failed to clone tcp stream"),
            stream_metadata,
        ));
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

    Some(read_handle)
}
