use std::sync::mpsc::Receiver;
use crate::types::package::{AppPackage, PackageMessage};
use crate::types::state::AppState;

fn draw_message(
    message: PackageMessage,
) {
    let msg = String::from_utf8_lossy(&message.msg).to_string();
    println!("[User: {}] {}", message.from, msg)
}

pub fn handle_packages(
    _app_state: AppState,
    rx: Receiver<AppPackage>,
) {
    while let Ok(package) = rx.recv() {
        match package {
            AppPackage::Message(message) => {
                draw_message(message)
            }
            AppPackage::NewConn(conn_data) => {
                println!("---new conn {}", conn_data.addr);
            }
        }
    }

    eprintln!("---channel hangup");
}
