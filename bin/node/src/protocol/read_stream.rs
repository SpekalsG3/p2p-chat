use std::net::{Shutdown, SocketAddr, TcpStream};
use std::time::{Duration, SystemTime};
use crate::commands::ProtocolCommand;
use crate::protocol::{
    frames::ProtocolMessage,
    node_info::NodeInfo,
    state::ProtocolState,
};
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage, MessagePackage};
use crate::utils::sss_triangle::sss_triangle;

pub fn protocol_read_stream(
    app_state: ProtocolState,
    addr: SocketAddr,
    mut stream: TcpStream, // should be cloned anyway bc otherwise `&mut` at `stream.read` will block whole application
) {
    loop {
        let message = ProtocolMessage::from_stream(&mut stream)
            .expect("---Failed to read stream");

        if message.is_none() {
            // stream has ended = host disconnected
            break;
        }
        let (message, frames_count) = message.unwrap();

        let lock = &mut *app_state.lock().expect("---Failed to get write lock");

        let streams = &mut lock.streams;
        let state = &mut lock.state;
        let data_id_states = &mut lock.data_id_states;

        for _ in 0..frames_count {
            state.next();
        }

        match message {
            ProtocolMessage::ConnInit { .. } => {
                unreachable!("Unexpected protocol message")
            }
            ProtocolMessage::ConnClosed => {
                let (ref mut stream, _) = streams
                    .get_mut(&addr)
                    .expect("Unknown address");

                stream.shutdown(Shutdown::Both).expect("Failed to shutdown");
                streams.remove(&addr);

                break;
            }
            ProtocolMessage::Data(id, data) => {
                if data_id_states.contains_key(&id) {
                    continue;
                }
                data_id_states.insert(id, ());

                let mut biggest_ping = 0;
                for (targ_addr, (ref mut stream, ref metadata)) in streams.iter_mut() {
                    if targ_addr == &addr {
                        continue
                    }
                    if metadata.ping > biggest_ping {
                        biggest_ping = metadata.ping;
                    }

                    ProtocolState::send_message(
                        state,
                        stream,
                        ProtocolMessage::Data(id, data.clone()),
                    )
                        .expect("Failed to write to stream");
                }

                {
                    let app_state = app_state.clone();
                    std::thread::spawn(move || {
                        let ping = if biggest_ping == 0 {
                            1
                        } else {
                            biggest_ping as u64 * 2 // x2 just to be sure
                        };
                        std::thread::sleep(Duration::from_millis(ping));

                        let mut lock = app_state.lock().expect("Failed to get write lock");
                        lock.data_id_states.remove(&id);
                    });
                }

                lock
                    .package_sender
                    .send(AppPackage::Message(MessagePackage {
                        from: addr,
                        msg: data,
                    }))
                    .expect("---Failed to send app package");
            }
            ProtocolMessage::NodeStatus(info) => {
                if streams.contains_key(&info.addr) {
                    continue;
                }
                if streams.len() < 4 { // todo: move as config variable
                    lock
                        .package_sender
                        .send(AppPackage::Alert(AlertPackage {
                            level: AlertPackageLevel::DEBUG,
                            msg: format!("Connecting to new node {} with src_ping of {}", info.addr, info.ping),
                        }))
                        .expect("---Failed to send app package");

                    lock
                        .command_sender
                        .send(ProtocolCommand::ClientConnect {
                            targ_addr: info.addr,
                            src_to_targ_ping: info.ping,
                            src_addr: addr,
                        })
                        .expect("---Failed to send NodeCommand");
                } else {
                    let mut biggest_ping = 0;
                    let mut worst_node = None;
                    for (addr, (stream, metadata)) in streams.iter() {
                        if metadata.ping > biggest_ping {
                            biggest_ping = metadata.ping;
                            worst_node = Some((addr, stream));
                        }
                    }

                    if let Some((r_addr, stream)) = worst_node {
                        lock
                            .package_sender
                            .send(AppPackage::Alert(AlertPackage {
                                level: AlertPackageLevel::DEBUG,
                                msg: format!("Changing nodes from {} to {}", r_addr, info.addr),
                            }))
                            .expect("---Failed to send app package");

                        // first connect in case there's unhandled problem, then disconnect
                        lock
                            .command_sender
                            .send(ProtocolCommand::ClientConnect {
                                targ_addr: info.addr,
                                src_to_targ_ping: info.ping,
                                src_addr: addr,
                            })
                            .expect("---Failed to send NodeCommand");

                        stream.shutdown(Shutdown::Both).expect("Failed to shutdown");
                        streams.remove(&r_addr.clone()); // clone is workaround for mutable with immutable references
                    }
                }
            }
            ProtocolMessage::Pong(info) => {
                let ping_info = if let Some(info) = info {
                    let src_ping = lock.streams.get(&info.addr).expect("src_addr should exist").1.ping;
                    Some((src_ping, info.ping))
                } else {
                    None
                };

                let (_, ref mut metadata) = lock
                    .streams
                    .get_mut(&addr)
                    .expect("Unknown address");

                if metadata.ping_started_at.is_none() {
                    continue; // haven't requested ping => cannot measure anything
                }

                let ping = SystemTime::now().duration_since(metadata.ping_started_at.unwrap()).unwrap().as_millis();
                if ping > 60_000 { // todo: move to constant
                    lock
                        .package_sender
                        .send(AppPackage::Alert(AlertPackage {
                            level: AlertPackageLevel::WARNING,
                            msg: format!("Ping with host {} is too big ({}). Disconnecting", addr, ping),
                        }))
                        .expect("---Failed to send app package");
                    stream.shutdown(Shutdown::Both).expect("---Failed to shutdown stream");
                    break;
                }
                let ping = ping as u16;

                metadata.ping = ping;
                metadata.ping_started_at = None;

                if let Some((src_ping, src_to_targ_ping)) = ping_info {
                    let angle = sss_triangle(src_ping, ping, src_to_targ_ping);
                    metadata.topology_rad = angle;

                    lock
                        .package_sender
                        .send(AppPackage::Alert(AlertPackage {
                            level: AlertPackageLevel::DEBUG,
                            msg: format!("Calculated angle of {}", angle),
                        }))
                        .expect("---Failed to send app package");
                }

                lock
                    .package_sender
                    .send(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::DEBUG,
                        msg: format!("Received pong, delay is {}", ping),
                    }))
                    .expect("---Failed to send app package");
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

                lock
                    .package_sender
                    .send(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::DEBUG,
                        msg: format!("Received ping from {}, sending pong with info {:?}", addr, info),
                    }))
                    .expect("---Failed to send app package");

                ProtocolState::send_message(
                    &mut lock.state,
                    &mut stream,
                    ProtocolMessage::Pong(info),
                )
                    .expect("Failed to send protocol to stream");
            }
        }
    }
}
