use std::net::{SocketAddr};
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use crate::core::client::start_client;
use crate::types::{
    state::ProtocolState,
    package::{AlertPackage, AlertPackageLevel, AppPackage},
};

pub enum ProtocolCommand {
    ClientConnect {
        targ_addr: SocketAddr,
        src_to_targ_ping: u16,
        src_addr: SocketAddr,
    },
    #[allow(unused)]
    ClientDisconnect(SocketAddr),
}

fn process_command(
    protocol_state: ProtocolState,
    command_receiver: Receiver<ProtocolCommand>,
) {
    let mut handles = vec![];

    while let Ok(command) = command_receiver.recv() {
        match command {
            ProtocolCommand::ClientConnect { targ_addr, src_addr, src_to_targ_ping } => {
                let h = start_client(
                    protocol_state.clone(),
                    targ_addr,
                    Some((src_addr, src_to_targ_ping)),
                );
                handles.extend(h);
            }
            ProtocolCommand::ClientDisconnect(addr) => {
                if let Err(e) = protocol_state.disconnect(addr) {
                    protocol_state
                        .read()
                        .package_sender.send(AppPackage::Alert(AlertPackage {
                            level: AlertPackageLevel::ERROR,
                            msg: format!("Failed to disconnect: {}", e)
                        }))
                        .expect("--Failed to send package")
                }
            }
        }
    }
}

pub fn command_processor(
    protocol_state: ProtocolState,
    command_receiver: Receiver<ProtocolCommand>,
) -> [JoinHandle<()>; 1] {
    let handle = std::thread::spawn(|| {
        process_command(protocol_state, command_receiver)
    });

    [handle]
}
