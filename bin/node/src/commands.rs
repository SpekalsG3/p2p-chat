use std::net::{SocketAddr};
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use crate::frontend::state::AppState;
use crate::protocol::client::start_client;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};

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
    app_state: AppState,
    command_receiver: Receiver<ProtocolCommand>,
) {
    let mut handles = vec![];

    while let Ok(command) = command_receiver.recv() {
        match command {
            ProtocolCommand::ClientConnect { targ_addr, src_addr, src_to_targ_ping } => {
                let h = start_client(
                    app_state.protocol_state.clone(),
                    targ_addr,
                    Some((src_addr, src_to_targ_ping)),
                );
                handles.extend(h);
            }
            ProtocolCommand::ClientDisconnect(addr) => {
                if let Err(e) = app_state.protocol_state.disconnect(addr) {
                    app_state.new_package(AppPackage::Alert(AlertPackage {
                        level: AlertPackageLevel::ERROR,
                        msg: format!("Failed to disconnect: {}", e)
                    }))
                }
            }
        }
    }
}

pub fn command_processor(
    app_state: AppState,
    command_receiver: Receiver<ProtocolCommand>,
) -> [JoinHandle<()>; 1] {
    let handle = std::thread::spawn(|| {
        process_command(app_state, command_receiver)
    });

    [handle]
}
