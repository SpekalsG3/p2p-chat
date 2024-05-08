use std::io::{stdout, Write};
use crate::frontend::send_message::send_message;
use crate::frontend::state::AppState;
use crate::types::ui::V100;

pub async fn handle_input(
    app_state: &AppState,
    str: &str,
) {
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
    } else {
        send_message(
            app_state,
            &str,
        ).await;
    }
}
