use std::net::{Shutdown, TcpStream};
use std::time::SystemTime;
use crate::commands::NodeCommand;
use crate::protocol::frames::ProtocolMessage;
use crate::protocol::node_info::NodeInfo;
use crate::protocol::start_pinging::start_pinging;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage, MessagePackage};
use crate::types::state::{AppState, MetaData};
use crate::utils::sss_triangle::sss_triangle;

pub fn protocol_read_stream(
    app_state: AppState,
    mut stream: TcpStream, // should be cloned anyway bc otherwise `&mut` at `stream.read` will block whole application
) {
    let addr;
    let pinging_handle;

    {
        let first_message = ProtocolMessage::from_stream(&mut stream)
            .expect("---Failed to read stream")
            .expect("---Should receive at least init message");

        addr = match first_message {
            ProtocolMessage::ConnInit { server_addr } => server_addr,
            _ => {
                unreachable!("Unexpected first message")
            }
        };

        let mut lock = app_state.write_lock().expect("---Failed to get write lock");

        lock
            .package_sender
            .send(AppPackage::Alert(AlertPackage {
                level: AlertPackageLevel::INFO,
                msg: format!("New join from {}", addr),
            }))
            .expect("---Failed to send app package");

        // todo: it's hardcode, provide choice to the user to change rooms
        AppState::set_selected_room(&mut lock, Some(addr));

        let mut conn_metadata = MetaData {
            ping: 0,
            ping_started_at: None,
            topology_rad: 0_f32,
            connected_to: vec![],
        };

        {
            let another_conn = lock.streams.iter().find(|(k, _)| !k.eq(&&addr));

            if let Some((targ_addr, (_, targ_metadata))) = another_conn {
                lock
                    .package_sender
                    .send(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::DEBUG,
                        msg: format!("Sending info about another node {}", targ_addr),
                    }))
                    .expect("---Failed to send app package");

                ProtocolMessage::NodeInfo(
                    NodeInfo::new(targ_addr.clone(), targ_metadata.ping)
                )
                    .send_to_stream(&mut stream)
                    .expect("---Failed to send protocol message");

                conn_metadata.connected_to.push(targ_addr.clone());
            }
        }

        lock.streams.insert(addr, (
            stream.try_clone().expect("---Failed to clone tcp stream"),
            conn_metadata
        ));

        {
            let app_state = app_state.clone();
            pinging_handle = std::thread::spawn(move || {
                start_pinging(app_state, addr)
            })
        }
    };

    loop {
        let message = ProtocolMessage::from_stream(&mut stream)
            .expect("---Failed to read stream");

        if message.is_none() {
            // stream has ended = host disconnected
            break;
        }
        let message = message.unwrap();

        match message {
            ProtocolMessage::ConnInit { .. } => {
                unreachable!("Unexpected protocol message")
            }
            ProtocolMessage::ConnClosed => {
                let mut lock = app_state.write_lock().expect("---Failed to get write lock");

                let (ref mut stream, _) = lock
                    .streams
                    .get_mut(&addr)
                    .expect("Unknown address");

                stream.shutdown(Shutdown::Both).expect("Failed to shutdown");
                lock.streams.remove(&addr);

                break;
            }
            ProtocolMessage::Data(data) => {
                let lock = app_state.read_lock().expect("---Failed to get write lock");
                lock
                    .package_sender
                    .send(AppPackage::Message(MessagePackage {
                        from: addr,
                        msg: data,
                    }))
                    .expect("---Failed to send app package");
            }
            ProtocolMessage::NodeInfo(info) => {
                let lock = app_state.read_lock().expect("---Failed to get write lock");

                if lock.streams.len() < 4 { // todo: move as config variable
                    lock
                        .package_sender
                        .send(AppPackage::Alert(AlertPackage {
                            level: AlertPackageLevel::DEBUG,
                            msg: format!("Connecting to new node {} with src_ping of {}", info.addr, info.ping),
                        }))
                        .expect("---Failed to send app package");

                    lock
                        .command_sender
                        .send(NodeCommand::ClientConnect {
                            targ_addr: info.addr,
                            src_to_targ_ping: info.ping,
                            src_addr: addr,
                        })
                        .expect("---Failed to send NodeCommand");
                } else {
                    // todo: if ping is lower then biggest latency we have, then disconnect and connect to that one
                }
            }
            ProtocolMessage::Pong(info) => {
                let mut lock = app_state.write_lock().expect("---Failed to get write lock");

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

                if let Some((src_ping, targ_ping)) = ping_info {
                    let angle = sss_triangle(src_ping, ping, targ_ping);
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
                let lock = app_state.read_lock().expect("---Failed to get write lock");
                let (_, metadata) = lock.streams.get(&addr).expect("entry should exist");

                let info = {
                    if let Some(targ_addr) = metadata.connected_to.get(0) {
                        let (_, metadata) = lock
                            .streams
                            .get(&targ_addr)
                            .expect("we should know about it bc `targ_addr` knows about us bc we connected to him");

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
                        msg: format!("Received ping, sending pong with info {:?}", info),
                    }))
                    .expect("---Failed to send app package");

                ProtocolMessage::Pong(info)
                    .send_to_stream(&mut stream)
                    .expect("Failed to send protocol to stream");
            }
        }
    }

    pinging_handle.join().expect("Should exit gracefully");
}
