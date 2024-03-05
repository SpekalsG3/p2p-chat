use std::sync::mpsc::Receiver;
use crate::frontend::state::AppState;
use crate::types::package::{AlertPackageLevel, AppPackage};

pub fn handle_packages(
    app_state: AppState,
    package_receiver: Receiver<AppPackage>,
) {
    while let Ok(package) = package_receiver.recv() {
        app_state.new_package(package);
    }

    app_state.ui.new_message(&format!("System: {}", AlertPackageLevel::INFO), "channel hangup");
}
