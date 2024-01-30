use std::sync::mpsc::Receiver;
use crate::types::package::{AlertPackageLevel, AppPackage};
use crate::types::state::AppState;
use crate::utils::ui::UITerminal;

pub fn handle_packages(
    _app_state: AppState,
    ui: UITerminal,
    rx: Receiver<AppPackage>,
) {
    while let Ok(package) = rx.recv() {
        match package {
            AppPackage::Message(message) => {
                let msg = String::from_utf8_lossy(&message.msg).to_string();
                ui.new_message(&format!("User: {}", message.from), &msg);
            }
            AppPackage::Alert(alert) => {
                ui.new_message(&format!("System: {}", alert.level), &alert.msg);
            }
        }
    }

    ui.new_message(&format!("System: {}", AlertPackageLevel::INFO), "channel hangup");
}
