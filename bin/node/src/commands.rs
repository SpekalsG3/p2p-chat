use std::net::{Shutdown, SocketAddr};
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use crate::client::start_client;
use crate::types::package::{AlertPackage, AlertPackageLevel, AppPackage};
use crate::types::state::AppState;

pub enum NodeCommand {
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
    command_receiver: Receiver<NodeCommand>,
) {
    let mut handles = vec![];

    while let Ok(command) = command_receiver.recv() {
        match command {
            NodeCommand::ClientConnect { targ_addr, src_addr, src_to_targ_ping } => {
                let h = start_client(
                    app_state.clone(),
                    targ_addr,
                    Some((src_addr, src_to_targ_ping)),
                );
                handles.extend(h);
            }
            NodeCommand::ClientDisconnect(addr) => {
                let mut lock = app_state.write_lock().expect("---Failed to get write lock");
                match lock.streams.get_mut(&addr) {
                    Some((s, _)) => {
                        s.shutdown(Shutdown::Both).expect("---Failed to shutdown the stream");
                        lock.streams.remove(&addr);
                    }
                    None => {
                        lock
                            .package_sender
                            .send(AppPackage::Alert(AlertPackage {
                                level: AlertPackageLevel::WARNING,
                                msg: format!("Stream for address {} does not exist", addr)
                            }))
                            .expect("---Failed to send app package");
                    }
                }
            }
        }
    }
}

pub fn command_processor(
    app_state: AppState,
    command_receiver: Receiver<NodeCommand>,
) -> [JoinHandle<()>; 1] {
    let handle = std::thread::spawn(|| {
        process_command(app_state, command_receiver)
    });

    [handle]
}
