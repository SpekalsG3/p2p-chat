use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;
use anyhow::{anyhow, bail, Context, Result};
use crate::types::package::AppPackage;
use crate::utils::ui::UiNoHistory;

pub(crate) struct AppStateInnerRef {
    ui: UiNoHistory,
}
pub(crate) struct AppStateInnerMut {
    pub(crate) selected_room: Option<SocketAddr>, // or store TcpStream and delete `streams`
    pub(crate) streams: HashMap<SocketAddr, TcpStream>,
    main_sender: Sender<AppPackage>,
}
pub(crate) struct AppStateInner {
    r: AppStateInnerRef,
    pub(crate) m: RwLock<AppStateInnerMut>,
}

pub struct AppState(pub(crate) Arc<AppStateInner>);

impl AppState {
    pub fn new(
        sender: Sender<AppPackage>,
    ) -> Self {
        Self(Arc::new(AppStateInner {
            r: AppStateInnerRef {
                ui: UiNoHistory::new(),
            },
            m: RwLock::new(AppStateInnerMut {
                selected_room: None,
                streams: Default::default(),
                main_sender: sender,
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
        lock.main_sender.send(package).context("---Failed to send app message")
    }

    pub fn ui(&self) -> &UiNoHistory {
        &self.0.r.ui
    }

    pub fn send_stream_message(&self, addr: &SocketAddr, message: &[u8]) -> Result<()> {
        let mut lock = self.0.m.write().map_err(|e| anyhow!("---Failed to acquire read lock: {}", e.to_string()))?;
        let mut stream = match lock.streams.get(addr) {
            Some(s) => s,
            None => {
                bail!("No stream for that address");
            }
        };

        stream.write(message).map_err(|e| anyhow!("---Failed to write to stream: {}", e.to_string()))?;

        Ok(())
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
