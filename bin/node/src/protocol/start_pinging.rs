use std::net::{Shutdown, SocketAddr};
use std::time::{Duration, SystemTime};
use crate::protocol::frames::ProtocolMessage;
use crate::types::state::AppState;

const PING_INTERVAL: u64 = 2 * 60; // 2 minutes

pub fn start_pinging(
    app_state: AppState,
    addr: SocketAddr,
) {
    loop {
        {
            let mut lock = app_state.write_lock().expect("---Failed to acquire write lock");
            let (ref mut stream, ref mut metadata) = lock
                .streams
                .get_mut(&addr)
                .expect("Unknown address");

            let now = SystemTime::now();

            if metadata.ping_started_at.is_some() {
                // means host did not respond to last ping = host is dead

                stream.shutdown(Shutdown::Both).expect("shutdown failed");
                lock.streams.remove(&addr);
                break;
            }

            metadata.ping_started_at = Some(now);
            ProtocolMessage::Ping
                .send_to_stream(stream)
                .expect("---Failed send frame");
        }

        // in the end because we want to start pinging right away
        std::thread::sleep(Duration::from_secs(PING_INTERVAL));
    }
}
