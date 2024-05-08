use std::net::{SocketAddr};
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
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

async fn process_command(
    protocol_state: ProtocolState,
    mut command_receiver: Receiver<ProtocolCommand>,
) {
    let mut handles = vec![];

    while let Some(command) = command_receiver.recv().await {
        match command {
            ProtocolCommand::ClientConnect { targ_addr, src_addr, src_to_targ_ping } => {
                let h = start_client(
                    protocol_state.clone(),
                    targ_addr,
                    Some((src_addr, src_to_targ_ping)),
                ).await;
                handles.extend(h);
            }
            ProtocolCommand::ClientDisconnect(addr) => {
                // todo
                // if let Err(e) = protocol_state.disconnect(addr).await {
                //     protocol_state
                //         .read()
                //         .package_sender.send(AppPackage::Alert(AlertPackage {
                //             level: AlertPackageLevel::ERROR,
                //             msg: format!("Failed to disconnect: {}", e)
                //         }))
                //         .await
                //         .expect("--Failed to send package")
                // }
            }
        }
    }
}

pub fn command_processor(
    protocol_state: ProtocolState,
    command_receiver: Receiver<ProtocolCommand>,
) -> [JoinHandle<()>; 1] {
    let handle = tokio::spawn(
        process_command(protocol_state, command_receiver)
    );

    [handle]
}
