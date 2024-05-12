use std::net::SocketAddr;
use std::time::{Duration, SystemTime};
use crate::core::{
    commands::ProtocolCommand,
    frames::ProtocolMessage,
    node_info::NodeInfo,
};
use crate::core::stream::types::StreamAction;
use crate::types::{
    state::ProtocolState,
    package::{AlertPackage, AlertPackageLevel, AppPackage, MessagePackage},
};
use crate::utils::sss_triangle::sss_triangle;

pub async fn read_message(
    protocol_state: &ProtocolState,
    addr: SocketAddr,
    message: Option<(ProtocolMessage, usize)>,
) -> StreamAction {
    if message.is_none() {
        // stream has ended = host disconnected
        return StreamAction::InitiateDisconnect;
    }
    let (message, _) = message.unwrap();

    let lock = &mut *protocol_state.lock().await;

    let streams = &mut lock.streams;
    let state = &mut lock.state;
    let data_id_states = &mut lock.data_id_states;

    state.next();

    match message {
        ProtocolMessage::ConnInit { .. } => {
            unreachable!("Unexpected protocol message")
        }
        ProtocolMessage::ConnClosed => {
            return StreamAction::AcceptDisconnect;
        }
        ProtocolMessage::Data(id, data) => {
            if data_id_states.contains_key(&id) {
                return StreamAction::None;
            }
            data_id_states.insert(id, ());

            let mut biggest_ping = 0;
            for (targ_addr, (channel, ref metadata)) in streams.iter_mut() {
                if targ_addr == &addr {
                    continue
                }
                if metadata.ping > biggest_ping {
                    biggest_ping = metadata.ping;
                }

                state.next();
                channel
                    .send(StreamAction::Send(ProtocolMessage::Data(id, data.clone())))
                    .await
                    .expect("Failed to send StreamRequest");
            }

            { // don't need to store the id longer than the largest ping between peers
                let app_state = protocol_state.clone();
                tokio::spawn(async move {
                    let ping = if biggest_ping == 0 {
                        1
                    } else {
                        biggest_ping as u64 * 2 // x2 just to be sure
                    };
                    tokio::time::sleep(Duration::from_millis(ping)).await;

                    let mut lock = app_state.lock().await;
                    lock.data_id_states.remove(&id);
                });
            }

            protocol_state
                .read()
                .package_sender
                .send(AppPackage::Message(MessagePackage {
                    from: addr,
                    msg: data,
                }))
                .await
                .expect("---Failed to send app package");
            return StreamAction::None;
        }
        ProtocolMessage::NodeStatus(info) => {
            if streams.contains_key(&info.addr) {
                return StreamAction::None;
            }
            if streams.len() < 4 { // todo: move as config variable
                protocol_state
                    .read()
                    .package_sender
                    .send(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::DEBUG,
                        msg: format!("Connecting to new node {} with src_ping of {}", info.addr, info.ping),
                    }))
                    .await
                    .expect("---Failed to send app package");

                lock
                    .command_sender
                    .send(ProtocolCommand::ClientConnect {
                        targ_addr: info.addr,
                        src_to_targ_ping: info.ping,
                        src_addr: addr,
                    })
                    .await
                    .expect("---Failed to send NodeCommand");
            } else {
                let mut biggest_ping = 0;
                let mut worst_node = None;
                for (addr, (channel, metadata)) in streams.iter_mut() {
                    if metadata.ping > biggest_ping {
                        biggest_ping = metadata.ping;
                        worst_node = Some((addr, channel));
                    }
                }

                if let Some((r_addr, channel)) = worst_node {
                    protocol_state
                        .read()
                        .package_sender
                        .send(AppPackage::Alert(AlertPackage {
                            level: AlertPackageLevel::DEBUG,
                            msg: format!("Changing nodes from {} to {}", r_addr, info.addr),
                        }))
                        .await
                        .expect("---Failed to send app package");

                    // first connect in case there's unhandled problem, then disconnect
                    lock
                        .command_sender
                        .send(ProtocolCommand::ClientConnect {
                            targ_addr: info.addr,
                            src_to_targ_ping: info.ping,
                            src_addr: addr,
                        })
                        .await
                        .expect("---Failed to send NodeCommand");

                    channel
                        .send(StreamAction::InitiateDisconnect)
                        .await
                        .expect("Failed to send StreamRequest");
                }
            }
            return StreamAction::None;
        }
        ProtocolMessage::Pong(info) => {
            let ping_info = if let Some(info) = info {
                let src_ping = lock.streams.get(&info.addr).expect("src_addr should exist").1.ping;
                Some((src_ping, info.ping))
            } else {
                None
            };

            let (_, metadata) = lock
                .streams
                .get_mut(&addr)
                .expect("Unknown address");

            if metadata.ping_started_at.is_none() {
                // haven't requested ping => cannot measure anything
                return StreamAction::None;
            }

            let ping = SystemTime::now().duration_since(metadata.ping_started_at.unwrap()).unwrap().as_millis();
            if ping > 60_000 { // todo: move to constant
                protocol_state
                    .read()
                    .package_sender
                    .send(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::WARNING,
                        msg: format!("Ping with host {} is too big ({}). Disconnecting", addr, ping),
                    }))
                    .await
                    .expect("---Failed to send app package");
                return StreamAction::InitiateDisconnect;
            }
            let ping = ping as u16;

            metadata.ping = ping;
            metadata.ping_started_at = None;

            if let Some((src_ping, src_to_targ_ping)) = ping_info {
                let angle = sss_triangle(src_ping, ping, src_to_targ_ping);
                metadata.topology_rad = angle;

                protocol_state
                    .read()
                    .package_sender
                    .send(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::DEBUG,
                        msg: format!("Calculated angle of {}", angle),
                    }))
                    .await
                    .expect("---Failed to send app package");
            }

            protocol_state
                .read()
                .package_sender
                .send(AppPackage::Alert(AlertPackage {
                    level: AlertPackageLevel::DEBUG,
                    msg: format!("Received pong, delay is {}", ping),
                }))
                .await
                .expect("---Failed to send app package");
            return StreamAction::None;
        }
        ProtocolMessage::Ping => {
            let (_, metadata) = lock.streams.get(&addr).expect("entry should exist");

            let info = {
                if let Some(targ_addr) = metadata.knows_about.get(0) {
                    let (_, metadata) = lock
                        .streams
                        .get(&targ_addr)
                        .expect("we should know about it bc `targ_addr` knows about us bc we connected to it");

                    Some(
                        NodeInfo::new(targ_addr.clone(), metadata.ping)
                    )
                } else {
                    None
                }
            };

            protocol_state
                .read()
                .package_sender
                .send(AppPackage::Alert(AlertPackage {
                    level: AlertPackageLevel::DEBUG,
                    msg: format!("Received ping from {}, sending pong with info {:?}", addr, info),
                }))
                .await
                .expect("---Failed to send app package");

            lock.state.next();
            return StreamAction::Send(ProtocolMessage::Pong(info));
        }
    }
}

