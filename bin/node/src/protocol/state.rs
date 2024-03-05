use std::collections::HashMap;
use std::io::Write;
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc::Sender;
use std::time::SystemTime;
use anyhow::{anyhow, Context, Result};
use crate::commands::ProtocolCommand;
use crate::protocol::frames::ProtocolMessage;
use crate::types::package::AppPackage;
use crate::utils::prng::{Splitmix64, Xoshiro256ss};

#[derive(Debug)]
pub(super) struct StreamMetadata {
    pub(super) ping: u16, // in milliseconds but we check that ping is less then 60000 so it can fit
    pub(super) ping_started_at: Option<SystemTime>,
    pub(super) topology_rad: f32, // angel relative to the first connection, used to determine who's closer to another user
    // vec of address this node knows about for any cross-referencing
    // (like for topology_rad or to find the path to specific node)
    pub(super) knows_about: Vec<SocketAddr>,
}

impl StreamMetadata {
    pub fn new() -> Self {
        Self {
            ping: 0,
            ping_started_at: None,
            topology_rad: 0_f32,
            knows_about: vec![],
        }
    }
}

pub(super) struct AppStateInnerMut {
    pub(super) server_addr: SocketAddr,
    pub(super) command_sender: Sender<ProtocolCommand>,
    pub(super) package_sender: Sender<AppPackage>,
    pub(super) streams: HashMap<SocketAddr, (TcpStream, StreamMetadata)>,
    pub(super) state: Xoshiro256ss,
    pub(super) data_id_states: HashMap<u64, ()>,
}
pub(super) struct AppStateInner {
    m: Mutex<AppStateInnerMut>,
}

pub struct ProtocolState(pub(super) Arc<AppStateInner>);

impl ProtocolState {
    pub fn new(
        server_addr: SocketAddr,
        command_sender: Sender<ProtocolCommand>,
        package_sender: Sender<AppPackage>,
        seed: u64,
    ) -> Self {
        Self(Arc::new(AppStateInner {
            m: Mutex::new(AppStateInnerMut {
                server_addr,
                command_sender,
                package_sender,
                streams: HashMap::new(),
                state: Splitmix64::new(seed).xorshift256ss(),
                data_id_states: HashMap::new(),
            }),
        }))
    }

    pub(super) fn lock(&self) -> Result<MutexGuard<AppStateInnerMut>> {
        self.0.m.lock().map_err(|e| anyhow!(e.to_string()))
    }

    pub(super) fn send_message(
        state: &mut Xoshiro256ss,
        stream: &mut TcpStream,
        message: ProtocolMessage,
    ) -> Result<()> {
        for chunk in message.into_frames()? {
            state.next();
            stream.write(&chunk).map_err(|e| anyhow!("---Failed to write to stream: {}", e.to_string()))?;
        }

        Ok(())
    }

    pub fn disconnect(&self, addr: SocketAddr) -> Result<()> {
        let mut lock = self.lock().context("---Failed to get write lock")?;

        let (stream, _) = lock.streams.get_mut(&addr).context("Addr does not exist")?;
        stream.shutdown(Shutdown::Both).context("Failed to shutdown the stream")?;
        lock.streams.remove(&addr);

        Ok(())
    }

    pub fn broadcast_data(&self, data: Vec<u8>) -> Result<()> {
        let lock = &mut *self.lock().expect("---Failed to acquire write lock");
        let streams = &mut lock.streams;
        let state = &mut lock.state;

        let id = state.next();

        for (_, (ref mut stream, _)) in streams.iter_mut() {
            ProtocolState::send_message(
                state,
                stream,
                ProtocolMessage::Data(id, data.clone()),
            )?;
        }

        Ok(())
    }
}

impl Clone for ProtocolState {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
