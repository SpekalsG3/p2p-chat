use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use crate::frontend::handle_input::handle_input;
use crate::frontend::handle_packages::handle_packages;
use crate::types::package::AppPackage;
use crate::types::state::AppState;
use crate::utils::ui::UITerminal;

mod handle_packages;
mod handle_input;
mod send_message;

pub fn setup_frontend(
    app_state: AppState,
    package_receiver: Receiver<AppPackage>,
) -> [JoinHandle<()>; 2] {
    let ui = UITerminal::new();

    [{
        let app_state = app_state.clone();
        let ui = ui.clone();
        std::thread::spawn(move || {
            handle_input(app_state, ui);
        })
    },
    {
        let app_state = app_state.clone();
        let ui = ui.clone();
        std::thread::spawn(move || {
            handle_packages(app_state, ui, package_receiver);
        })
    }]
}
