use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpStream};
use std::time::SystemTime;
use super::vars::{PROTOCOL_BUF_SIZE, ProtocolAction, ProtocolBufferType};
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
                        let mut lock = app_state.write_lock().expect("---Failed to get write lock");
                        match buf_type {
                            ProtocolBufferType::Data => {
                                AppState::send_package(
                                    &mut lock,
                                    AppPackage::Message(MessagePackage {
                                        from: addr,
                                        msg: buf.clone(),
                                    }),
                                )
                                    .expect("---failed to send msg through channel");
                            }
                            ProtocolBufferType::TopologyUpd => {
                                let mut iter = buf.clone().into_iter();
                                while let Some(t) = iter.next() {
                                    match t {
                                        1 => {
                                            let ip = Ipv4Addr::new(
                                                iter.next().unwrap(),
                                                iter.next().unwrap(),
                                                iter.next().unwrap(),
                                                iter.next().unwrap(),
                                            );
                                            let port = u16::from_be_bytes([
                                                iter.next().unwrap(),
                                                iter.next().unwrap(),
                                            ]);
                                            let addr = SocketAddr::new(IpAddr::V4(ip), port);
                                            // todo: send `global action`-like stuff to local channel to start new client closer to main-thread
                                            //  will be also reused for user-accessible commands `/connect` `/server`
                                        }
                                        _ => {
                                            panic!("Unknown byte of TopologyUpd Buffer")
                                        }
                                    }
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
                        metadata.ping = SystemTime::now().duration_since(metadata.ping_started_at.unwrap()).unwrap();
                    }
                }
            }
            Err(e) => {
                let mut lock = app_state.write_lock().expect("---Failed to get write lock");
                AppState::send_package(
                    &mut lock,
                    AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::ERROR,
                        msg: format!("Failed to read stream - {}", e),
                    }),
                )
                    .expect("---Failed to send package");
            }
        }
    }
}
