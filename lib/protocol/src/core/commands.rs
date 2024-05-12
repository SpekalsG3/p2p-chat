use std::net::{SocketAddr};
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use crate::core::client::start_client;
use crate::core::stream::types::StreamAction;
use crate::types::state::ProtocolState;

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
                if let Some((channel, _)) = protocol_state.lock().await.streams.get(&addr) {
                    channel
                        .send(StreamAction::InitiateDisconnect)
                        .await
                        .expect("--Failed to send request to disconnect")
                }
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
