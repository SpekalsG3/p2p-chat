use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use super::vars::{PROTOCOL_BUF_SIZE, ProtocolAction};
use super::decode_frame::protocol_decode_frame;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage, MessagePackage};
use crate::types::state::AppState;

pub fn protocol_read_stream(
    app_state: &AppState,
    addr: SocketAddr,
    mut stream: TcpStream,
) {
    let mut buf = Vec::new();

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
                    },
                    ProtocolAction::UseBuffer => {
                        app_state
                            .send_package(AppPackage::Message(MessagePackage {
                                from: addr,
                                msg: buf.clone(),
                            }))
                            .expect("---failed to send msg through channel");
                        buf.clear();
                    },
                    ProtocolAction::CloseConnection => {
                        break;
                    },
                    ProtocolAction::Send(s) => {
                        stream.write(&s).expect("Failed to write");
                    }
                }
            }
            Err(e) => {
                app_state
                    .send_package(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::ERROR,
                        msg: format!("Failed to read stream - {}", e),
                    }))
                    .expect("---Failed to send package");
            }
        }
    }
}
