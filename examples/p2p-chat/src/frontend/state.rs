use std::sync::Arc;
use protocol::types::{
    state::ProtocolState,
    package::AppPackage,
};
use crate::utils::ui::UITerminal;

pub struct AppStateInner {
    pub protocol_state: ProtocolState, // this is used to interact with the protocol
    pub ui: UITerminal, // this is accessed only by frontend
}

impl AppStateInner {
    pub fn new_package(&self, package: AppPackage) {
        match package {
            AppPackage::Message(message) => {
                let msg = String::from_utf8_lossy(&message.msg).to_string();
                self.ui.new_message(&format!("User: {}", message.from), &msg);
            }
            AppPackage::Alert(_alert) => {
                // self.ui.new_message(&format!("System: {}", alert.level), &alert.msg);
                // todo: write macro to wrap sending packages and ignore `level: DEBUG` in release mode
                return;
            }
        }
    }
}

pub type AppState = Arc<AppStateInner>;
