use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::mpsc::Sender;
use anyhow::{anyhow, Result};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, MutexGuard};
use crate::core::{
    commands::ProtocolCommand,
    frames::ProtocolMessage,
};
use crate::types::package::AppPackage;
use crate::utils::prng::{Splitmix64, Xoshiro256ss};

pub enum StreamRequest {
    Disconnect,
    Send(ProtocolMessage)
}

#[derive(Debug)]
pub(crate) struct StreamMetadata {
    pub ping: u16, // in milliseconds but we check that ping is less then 60000 so it can fit
    pub ping_started_at: Option<SystemTime>,
    pub topology_rad: f32, // angel relative to the first connection, used to determine who's closer to another user
    // vec of address this node knows about for any cross-referencing
    // (like for topology_rad or to find the path to specific node)
    pub knows_about: Vec<SocketAddr>,
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

pub struct ProtocolStateInnerRead {
    pub server_addr: SocketAddr,
    pub package_sender: Sender<AppPackage>,
}
pub(crate) struct ProtocolStateInnerMut {
    pub command_sender: Sender<ProtocolCommand>,
    pub streams: HashMap<SocketAddr, (Sender<StreamRequest>, StreamMetadata)>,
    pub state: Xoshiro256ss,
    pub data_id_states: HashMap<u64, ()>,
}
pub struct ProtocolStateInner {
    r: ProtocolStateInnerRead,
    m: Mutex<ProtocolStateInnerMut>,
}

pub struct ProtocolState(pub Arc<ProtocolStateInner>);

impl ProtocolState {
    pub fn new(
        server_addr: SocketAddr,
        command_sender: Sender<ProtocolCommand>,
        package_sender: Sender<AppPackage>,
        seed: u64,
    ) -> Self {
        Self(Arc::new(ProtocolStateInner {
            r: ProtocolStateInnerRead {
                server_addr,
                package_sender,
            },
            m: Mutex::new(ProtocolStateInnerMut {
                command_sender,
                streams: HashMap::new(),
                state: Splitmix64::new(seed).xorshift256ss(),
                data_id_states: HashMap::new(),
            }),
        }))
    }

    pub fn read(&self) -> &ProtocolStateInnerRead {
        &self.0.r
    }

    pub(crate) async fn lock(&self) -> MutexGuard<ProtocolStateInnerMut> {
        self.0.m.lock().await
    }

    pub async fn send_message(
        stream: &mut TcpStream,
        message: ProtocolMessage,
    ) -> Result<()> {
        for chunk in message.into_frames()? {
            stream.write(&chunk).await.map_err(|e| anyhow!("---Failed to write to stream: {}", e.to_string()))?;
        }

        Ok(())
    }

    pub async fn broadcast_data(&self, data: Vec<u8>) -> Result<()> {
        let lock = &mut *self.lock().await;
        let streams = &mut lock.streams;
        let state = &mut lock.state;

        let id = state.next();

        for (_, (ref mut channel, _)) in streams.iter_mut() {
            channel
                .send(StreamRequest::Send(ProtocolMessage::Data(id, data.clone())))
                .await
                .expect("Failed to send data to stream request channel")
        }

        Ok(())
    }
}

impl Clone for ProtocolState {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
