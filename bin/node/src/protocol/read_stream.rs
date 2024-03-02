use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::time::SystemTime;
use crate::commands::NodeCommand;
use super::vars::{NodeInfo, PROTOCOL_BUF_SIZE, ProtocolAction, ProtocolBufferType};
use super::decode_frame::protocol_decode_frame;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage, MessagePackage};
use crate::types::state::AppState;

pub fn protocol_read_stream(
    app_state: AppState,
    addr: SocketAddr,
    mut stream: TcpStream, // should be cloned anyway bc otherwise `&mut` at `stream.read` will block whole application
) {
    let mut buf = Vec::new();
    let mut buf_type = ProtocolBufferType::Data;

    loop {
        let mut frame = [0; PROTOCOL_BUF_SIZE];
        match stream.read(&mut frame) {
            Ok(n) => {
                if n == 0 { // means socket is empty
                    break;
                }

                let res = protocol_decode_frame(&mut buf, frame)
                    .expect("Failed to parse protocol frame");
                match res {
                    ProtocolAction::None => {
                        continue
                    }
                    ProtocolAction::UpdateBufferType(t) => {
                        buf_type = t;
                        continue
                    }
                    ProtocolAction::UseBuffer => {
                        let lock = app_state.write_lock().expect("---Failed to get write lock");
                        match buf_type {
                            ProtocolBufferType::Data => {
                                lock
                                    .package_sender
                                    .send(AppPackage::Message(MessagePackage {
                                        from: addr,
                                        msg: buf.clone(),
                                    }))
                                    .expect("---Failed to send app package");
                            }
                            ProtocolBufferType::NodeInfo => {
                                let info = NodeInfo::from_bytes(buf.clone()).expect("---Failed to parse NodeInfo");

                                if lock.streams.len() < 4 { // todo: move as config variable
                                    lock
                                        .command_sender
                                        .send(NodeCommand::ClientConnect {
                                            src_addr: info.addr,
                                            src_ping: info.ping,
                                            targ: addr,
                                        })
                                        .expect("---Failed to send NodeCommand");
                                } else {
                                    // todo: if ping is lower then biggest latency we have, then disconnect and connect to that one
                                }
                            }
                        }
                        buf.clear();
                    }
                    ProtocolAction::CloseConnection => {
                        let mut lock = app_state.write_lock().expect("---Failed to get write lock");

                        let (ref mut stream, _) = lock
                            .streams
                            .get_mut(&addr)
                            .expect("Unknown address");

                        stream.shutdown(Shutdown::Both).expect("Failed to shutdown");
                        lock.streams.remove(&addr);

                        break;
                    },
                    ProtocolAction::Send(s) => {
                        stream.write(&s).expect("Failed to write");
                    }
                    ProtocolAction::ReceivedPong => {
                        let mut lock = app_state.write_lock().expect("---Failed to get write lock");

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

                        metadata.ping = ping as u16;
                    }
                }
            }
            Err(e) => {
                let lock = app_state.write_lock().expect("---Failed to get write lock");
                lock
                    .package_sender
                    .send(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::ERROR,
                        msg: format!("Failed to read stream - {}", e),
                    }))
                    .expect("---Failed to send app package");
            }
        }
    }
}
