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

        let app_state = app_state.clone();
        let addr = {
            app_state.0.m
                .read()
                .expect("---failed to take read lock")
                .selected_room
                .clone()
        };
        if let Some(addr) = addr {
            let ui = ui.clone();
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

    for h in handles {
        h.join().expect("---failed to join");
    }
}
