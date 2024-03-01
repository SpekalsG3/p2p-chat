use std::io::{stdin, stdout, Write};
use crate::frontend::send_message::send_message;
use crate::types::state::AppState;
use crate::types::ui::V100;
use crate::utils::ui::UITerminal;

pub fn handle_input(
    app_state: AppState,
    ui: UITerminal,
) {
    let mut handles = vec![];
    let stdin = stdin();

    loop {
        let mut str = String::new();
        stdin
            .read_line(&mut str)
            .expect("Failed to read line");
        if str.trim() == "" {
            let mut stdout = stdout().lock();
            stdout
                .write(format!(
                    "{}{}{}---\n\r>{}",
                    V100::GoLineUp(2),
                    V100::ClearLine,
                    V100::MoveWindowUp,
                    V100::ClearLineRight,
                ).as_bytes())
                .expect("Failed to write");
            stdout.flush().expect("failed to flush");
            continue
        }

        let lock = app_state.read_lock().expect("---Failed to acquire read lock");
        if let Some(addr) = AppState::get_selected_room(&lock) {
            let ui = ui.clone();
            let app_state = app_state.clone();

            let h = std::thread::spawn(move || {
                send_message(
                    app_state,
                    ui,
                    &str,
                    addr,
                )
            });

            handles.push(h); // later will add proper shutdown
        }
    }
}
