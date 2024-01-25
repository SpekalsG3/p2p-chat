use std::sync::mpsc::Receiver;
use crate::types::package::AppPackage;
use crate::types::state::AppState;

pub fn handle_packages(
    app_state: AppState,
    rx: Receiver<AppPackage>,
) {
    while let Ok(package) = rx.recv() {
        match package {
            AppPackage::Message(message) => {
                let msg = String::from_utf8_lossy(&message.msg).to_string();
                app_state.ui().new_message(&message.from.to_string(), &msg)
            }
            AppPackage::NewConn(conn_data) => {
                app_state.ui().system_message(&format!("new conn {}", conn_data.addr));
            }
        }
    }

    app_state.ui().system_message("channel hangup");
}
