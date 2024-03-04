use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::mpsc::Sender;
use std::time::SystemTime;
use anyhow::{anyhow, Result};
use crate::commands::NodeCommand;
use crate::protocol::frames::ProtocolMessage;
use crate::types::package::AppPackage;
use crate::utils::prng::{Splitmix64, Xoshiro256ss};

#[derive(Debug)]
pub struct MetaData {
    pub(crate) ping: u16, // in milliseconds but we check that ping is less then 60000 so it can fit
    pub(crate) ping_started_at: Option<SystemTime>,
    pub(crate) topology_rad: f32, // angel relative to the first connection, used to determine who's closer to another user
    // vec of address this node knows about for any cross-referencing
    // (like for topology_rad or to find the path to specific node)
    pub(crate) knows_about: Vec<SocketAddr>,
}

impl MetaData {
    pub fn new() -> Self {
        Self {
            ping: 0,
            ping_started_at: None,
            topology_rad: 0_f32,
            knows_about: vec![],
        }
    }
}

pub(crate) struct AppStateInnerRef {}
pub(crate) struct AppStateInnerMut {
    pub(crate) server_addr: SocketAddr,
    pub(crate) command_sender: Sender<NodeCommand>,
    pub(crate) package_sender: Sender<AppPackage>,
    pub(crate) streams: HashMap<SocketAddr, (TcpStream, MetaData)>,
    pub(crate) state: Xoshiro256ss,
    pub(crate) data_id_states: HashMap<u64, ()>,
}
pub(crate) struct AppStateInner {
    _r: AppStateInnerRef,
    m: RwLock<AppStateInnerMut>,
}

pub struct AppState(pub(crate) Arc<AppStateInner>);

impl AppState {
    pub fn new(
        server_addr: SocketAddr,
        command_sender: Sender<NodeCommand>,
        package_sender: Sender<AppPackage>,
        seed: u64,
    ) -> Self {
        Self(Arc::new(AppStateInner {
            _r: AppStateInnerRef {
            },
            m: RwLock::new(AppStateInnerMut {
                server_addr,
                command_sender,
                package_sender,
                streams: HashMap::new(),
                state: Splitmix64::new(seed).xorshift256ss(),
                data_id_states: HashMap::new(),
            }),
        }))
    }

    pub fn read_lock(&self) -> Result<RwLockReadGuard<'_, AppStateInnerMut>> {
        self.0.m.read().map_err(|e| anyhow!(e.to_string()))
    }

    pub fn write_lock(&self) -> Result<RwLockWriteGuard<'_, AppStateInnerMut>> {
        self.0.m.write().map_err(|e| anyhow!(e.to_string()))
    }

    pub fn send_message(
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
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
