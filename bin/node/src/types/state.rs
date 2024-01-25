use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;
use anyhow::{anyhow, Context, Result};
use crate::types::package::AppPackage;
use crate::utils::ui::UiNoHistory;

struct AppStateInnerRef {
    ui: UiNoHistory,
}
struct AppStateInnerMut {
    streams: HashMap<SocketAddr, TcpStream>,
    main_sender: Sender<AppPackage>,
}
struct AppStateInner {
    r: AppStateInnerRef,
    m: RwLock<AppStateInnerMut>,
}

pub struct AppState(Arc<AppStateInner>);

impl AppState {
    pub fn new(
        sender: Sender<AppPackage>,
    ) -> Self {
        Self(Arc::new(AppStateInner {
            r: AppStateInnerRef {
                ui: UiNoHistory::new(),
            },
            m: RwLock::new(AppStateInnerMut {
                streams: Default::default(),
                main_sender: sender,
            }),
        }))
    }

    pub fn add_stream(&self, addr: SocketAddr, stream: TcpStream) -> Result<()> {
        let mut lock = self.0.m.write().map_err(|e| anyhow!("---Failed to acquire read lock: {}", e.to_string()))?;
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
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
