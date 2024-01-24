use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;
use anyhow::{anyhow, Context, Result};
use crate::types::package::AppPackage;

struct AppStateInner {
    streams: HashMap<SocketAddr, TcpStream>,
    main_sender: Sender<AppPackage>,
}

pub struct AppState(Arc<RwLock<AppStateInner>>);

impl AppState {
    pub fn new(
        sender: Sender<AppPackage>,
    ) -> Self {
        Self(Arc::new(RwLock::new(AppStateInner {
            streams: Default::default(),
            main_sender: sender,
        })))
    }

    pub fn add_stream(&self, addr: SocketAddr, stream: TcpStream) -> Result<()> {
        let mut lock = self.0.write().map_err(|e| anyhow!("---Failed to acquire read lock: {}", e.to_string()))?;
        lock.streams.insert(addr, stream);
        Ok(())
    }

    pub fn send_package(&self, package: AppPackage) -> Result<()> {
        let lock = self.0.read().map_err(|e| anyhow!("---Failed to acquire read lock: {}", e.to_string()))?;
        lock.main_sender.send(package).context("---Failed to send app message")
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
