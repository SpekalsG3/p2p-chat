use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::select;
use tokio::sync::mpsc::Receiver;
use crate::frontend::handle_input::handle_input;
use crate::frontend::state::AppState;
use protocol::types::package::{AlertPackage, AlertPackageLevel, AppPackage};

mod handle_input;
mod send_message;
pub mod state;

pub async fn setup_frontend(
    app_state: AppState,
    mut package_receiver: Receiver<AppPackage>,
) {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    loop {
        select! {
            res = reader.read_line(&mut line) => {
                match res {
                    Ok(_) => {
                        handle_input(&app_state, &line).await;
                    },
                    Err(e) => {
                        app_state.new_package(AppPackage::Alert(AlertPackage {
                            level: AlertPackageLevel::ERROR,
                            msg: format!("Failed to read_line {}", e),
                        }));
                    }
                }
            },
            package = package_receiver.recv() => {
                if let Some(package) = package {
                    app_state.new_package(package);
                } else {
                    app_state.ui.new_message(&format!("System: {}", AlertPackageLevel::INFO), "channel hangup");
                    break;
                }
            }
        }
    }
}
