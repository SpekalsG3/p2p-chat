use std::net::{Shutdown, SocketAddr, TcpStream};
use std::thread::JoinHandle;
use std::time::SystemTime;
use crate::protocol::frames::ProtocolMessage;
use crate::protocol::read_stream::protocol_read_stream;
use crate::protocol::state::{ProtocolState, StreamMetadata};
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};
use crate::utils::sss_triangle::sss_triangle;

pub fn start_client(
    app_state: ProtocolState,
    addr: SocketAddr,
    src_info: Option<(SocketAddr, u16)>,
) -> Option<JoinHandle<()>> {
    let ping = SystemTime::now();
    let mut stream = TcpStream::connect(addr).expect("---Failed to connect");
    let ping = SystemTime::now().duration_since(ping).expect("Failed to calculate ping").as_millis();

    {
        let mut lock = app_state.lock().expect("---Failed to get write lock");

        let server_addr = lock.server_addr;
        ProtocolState::send_message(
            &mut lock.state,
            &mut stream,
            ProtocolMessage::ConnInit {
                server_addr,
            },
        )
            .expect("---Failed to write to stream");

        lock
            .package_sender
            .send(AppPackage::Alert(AlertPackage {
                level: AlertPackageLevel::DEBUG,
                msg: format!("Sent init message to {} with server_addr {}", addr, lock.server_addr),
            }))
            .expect("---Failed to send app package");

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

        let mut targ_metadata = StreamMetadata::new();
        targ_metadata.ping = ping;

        if let Some((src_addr, src_to_targ_ping)) = src_info {
            let src_ping = lock.streams.get_mut(&src_addr).expect("src_addr should exist").1.ping;

            let angle = sss_triangle(src_ping, ping, src_to_targ_ping);

            lock
                .package_sender
                .send(AppPackage::Alert(AlertPackage {
                    level: AlertPackageLevel::DEBUG,
                    msg: format!("Calculated angle of {} for {}", angle, src_addr),
                }))
                .expect("---Failed to send app package");

            targ_metadata.topology_rad = angle;
            targ_metadata.knows_about.push(src_addr.clone());
        }

        lock.streams.insert(addr, (
            stream.try_clone().expect("---Failed to clone tcp stream"),
            targ_metadata,
        ));

        lock
            .package_sender
            .send(AppPackage::Alert(AlertPackage {
                level: AlertPackageLevel::DEBUG,
                msg: format!("Connected with delay of {}", ping),
            }))
            .expect("---Failed to send app package");
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
