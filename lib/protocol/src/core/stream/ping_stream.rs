use std::net::SocketAddr;
use std::time::SystemTime;
use crate::core::frames::ProtocolMessage;
use crate::core::stream::types::StreamAction;
use crate::types::{
    state::ProtocolState,
    package::{AlertPackage, AlertPackageLevel, AppPackage},
};

pub const PING_INTERVAL: u64 = 2 * 60; // 2 minutes

pub async fn ping_action(
    protocol_state: &ProtocolState,
    addr: SocketAddr,
) -> StreamAction {
    let lock = &mut *protocol_state.lock().await;
    let streams = &mut lock.streams;
    let state = &mut lock.state;

    let (_, ref mut metadata) = streams
        .get_mut(&addr)
        .expect("Unknown address");

    let now = SystemTime::now();

    if metadata.ping_started_at.is_some() {
        // means host did not respond to last ping = host is dead

        return StreamAction::Disconnect;
    }

    metadata.ping_started_at = Some(now);
    state.next();

    protocol_state
        .read()
        .package_sender
        .send(AppPackage::Alert(AlertPackage {
            level: AlertPackageLevel::DEBUG,
            msg: "Sending ping".to_string(),
        }))
        .await
        .expect("---Failed to send app package");

    return StreamAction::Send(ProtocolMessage::Ping);
}
