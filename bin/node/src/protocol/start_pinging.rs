use std::net::{Shutdown, SocketAddr};
use std::time::{Duration, SystemTime};
use crate::protocol::frames::ProtocolMessage;
use crate::protocol::state::ProtocolState;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};

const PING_INTERVAL: u64 = 2 * 60; // 2 minutes

pub fn start_pinging(
    app_state: ProtocolState,
    addr: SocketAddr,
) {
    std::thread::sleep(Duration::from_secs(1)); // todo: remove, used only for debug

    loop {
        {
            let lock = &mut *app_state.lock().expect("---Failed to acquire write lock");
            let streams = &mut lock.streams;
            let state = &mut lock.state;

            let (ref mut stream, ref mut metadata) = streams
                .get_mut(&addr)
                .expect("Unknown address");

            let now = SystemTime::now();

            if metadata.ping_started_at.is_some() {
                // means host did not respond to last ping = host is dead

                stream.shutdown(Shutdown::Both).expect("shutdown failed");
                streams.remove(&addr);

                break;
            }

            metadata.ping_started_at = Some(now);
            ProtocolState::send_message(
                state,
                stream,
                ProtocolMessage::Ping
            )
                .expect("---Failed send frame");

            lock
                .package_sender
                .send(AppPackage::Alert(AlertPackage {
                    level: AlertPackageLevel::DEBUG,
                    msg: "Sent ping".to_string(),
                }))
                .expect("---Failed to send app package");
        }

        // in the end because we want to start pinging right away
        std::thread::sleep(Duration::from_secs(PING_INTERVAL));
    }
}
