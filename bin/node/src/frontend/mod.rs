use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use crate::frontend::handle_input::handle_input;
use crate::frontend::handle_packages::handle_packages;
use crate::frontend::state::AppState;
use crate::types::package::AppPackage;

mod handle_packages;
mod handle_input;
mod send_message;
pub mod state;

pub fn setup_frontend(
    app_state: AppState,
    package_receiver: Receiver<AppPackage>,
) -> [JoinHandle<()>; 2] {
    [{
        let app_state = app_state.clone();
        std::thread::spawn(move || {
            handle_input(app_state);
        })
    },
    {
        let app_state = app_state.clone();
        std::thread::spawn(move || {
            handle_packages(
                app_state,
                package_receiver,
            );
        })
    }]
}
