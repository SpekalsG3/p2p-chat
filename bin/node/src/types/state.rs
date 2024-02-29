use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;
use anyhow::{anyhow, bail, Context, Result};
use crate::protocol::encode_frame_data::protocol_encode_frame_data;
use crate::protocol::vars::PROT_OPCODE_DATA;
use crate::types::package::AppPackage;

pub(crate) struct AppStateInnerRef {
}
pub(crate) struct AppStateInnerMut {
    pub(crate) selected_room: Option<SocketAddr>, // or store TcpStream and delete `streams`
    pub(crate) streams: HashMap<SocketAddr, TcpStream>,
    packager_sender: Sender<AppPackage>,
}
pub(crate) struct AppStateInner {
    _r: AppStateInnerRef,
    pub(crate) m: RwLock<AppStateInnerMut>,
}

pub struct AppState(pub(crate) Arc<AppStateInner>);

impl AppState {
    pub fn new(
        package_sender: Sender<AppPackage>,
    ) -> Self {
        Self(Arc::new(AppStateInner {
            _r: AppStateInnerRef {
            },
            m: RwLock::new(AppStateInnerMut {
                selected_room: None,
                streams: Default::default(),
                packager_sender: package_sender,
            }),
        }))
    }

    pub fn add_stream(&self, addr: SocketAddr, stream: TcpStream) -> Result<()> {
        let mut lock = self.0.m.write().map_err(|e| anyhow!("---Failed to acquire write lock: {}", e.to_string()))?;
        lock.streams.insert(addr, stream);
        Ok(())
    }

    pub fn send_package(&self, package: AppPackage) -> Result<()> {
        let lock = self.0.m.read().map_err(|e| anyhow!("---Failed to acquire read lock: {}", e.to_string()))?;
        lock.packager_sender.send(package).context("---Failed to send app message")
    }

    pub fn send_stream_message(&self, addr: &SocketAddr, message: &[u8]) -> Result<()> {
        let chunks = protocol_encode_frame_data(PROT_OPCODE_DATA, message);

        let lock = self.0.m.write().map_err(|e| anyhow!("---Failed to acquire read lock: {}", e.to_string()))?;
        let mut stream = match lock.streams.get(addr) {
            Some(s) => s,
            None => {
                bail!("No stream for that address");
            }
        };

        for chunk in chunks {
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
