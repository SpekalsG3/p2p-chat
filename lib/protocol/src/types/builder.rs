use std::net::SocketAddr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;
use crate::core::client::start_client;
use crate::core::commands::{command_processor, ProtocolCommand};
use crate::core::server::handle_connection::start_server;
use crate::types::{
    state::ProtocolState,
    package::AppPackage,
};

pub struct ProtocolBuilder {
    state: ProtocolState,
    command_receiver: Receiver<ProtocolCommand>,
    handles: Vec<JoinHandle<()>>,
}

impl ProtocolBuilder {
    pub fn new(
        server_addr: SocketAddr,
        package_sender: Sender<AppPackage>,
        rng_seed: u64,
    ) -> Self {
        let (command_sender, command_receiver) = channel();

        let state = ProtocolState::new(
            server_addr,
            command_sender,
            package_sender,
            rng_seed,
        );

        Self {
            state,
            command_receiver,
            handles: vec![],
        }
    }

    pub fn set_client(
        &mut self,
        client_addr: SocketAddr,
    ) {
        let protocol_state = self.state.clone();
        let handle = start_client(protocol_state, client_addr, None);
        self.handles.extend(handle);
    }

    pub fn build(mut self) -> (ProtocolState, Vec<JoinHandle<()>>) {
        self.handles.extend(command_processor(
            self.state.clone(),
            self.command_receiver, // this is a bridge from application to protocol
        ));

        let protocol_state = self.state.clone();
        if let Some(handle) = start_server(protocol_state, self.state.read().server_addr) {
            self.handles.push(handle);
        }

        (self.state, self.handles)
    }
}
