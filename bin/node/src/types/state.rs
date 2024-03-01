use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;
use anyhow::{anyhow, bail, Context, Result};
use crate::protocol::encode_frame_data::ProtocolFrame;
use crate::types::package::AppPackage;

pub(crate) struct AppStateInnerRef {}
pub(crate) struct AppStateInnerMut {
    package_sender: Sender<AppPackage>,
    selected_room: Option<SocketAddr>,
    streams: HashMap<SocketAddr, TcpStream>,
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
                package_sender,
                selected_room: None,
                streams: HashMap::new(),
            }),
        }))
    }

    pub fn get_selected_room(&self) -> Result<Option<SocketAddr>> {
        Ok(
            self.0.m
                .read()
                .map_err(|e| anyhow!("---Failed to acquire write lock: {}", e.to_string()))?
                .selected_room
        )
    }

    pub fn set_selected_room(&self, room: Option<SocketAddr>) -> Result<()> {
        let mut lock = self.0.m.write().map_err(|e| anyhow!("---Failed to acquire write lock: {}", e.to_string()))?;
        lock.selected_room = room;
        Ok(())
    }

    pub fn add_stream(&self, addr: SocketAddr, stream: TcpStream) -> Result<()> {
        let mut lock = self.0.m.write().map_err(|e| anyhow!("---Failed to acquire write lock: {}", e.to_string()))?;
        lock.streams.insert(addr, stream);
        Ok(())
    }

    pub fn send_package(&self, package: AppPackage) -> Result<()> {
        let lock = self.0.m.read().map_err(|e| anyhow!("---Failed to acquire read lock: {}", e.to_string()))?;
        lock.package_sender.send(package).context("---Failed to send app message")
    }

    pub fn send_stream_message(&self, addr: &SocketAddr, frame: ProtocolFrame) -> Result<()> {
        let lock = self.0.m.write().map_err(|e| anyhow!("---Failed to acquire read lock: {}", e.to_string()))?;
        let mut stream = match lock.streams.get(addr) {
            Some(s) => s,
            None => {
                bail!("No stream for that address");
            }
        };

        for chunk in frame {
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
