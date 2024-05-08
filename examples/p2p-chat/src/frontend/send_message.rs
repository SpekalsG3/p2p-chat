use std::io::{stdout, Write};
use crate::frontend::state::AppState;
use crate::types::ui::V100;

pub async fn send_message(
    app_state: &AppState,
    message: &str,
) {
    let index = {
        let mut stdout = stdout().lock();
        stdout
            .write(format!(
                "{}",
                V100::GoLineUp(1),
            ).as_bytes())
            .expect("Failed to write");

        let index = app_state.ui.new_message("YOU", message);

        stdout
            .write(format!(
                "{}{}>",
                V100::GoLineDown(1),
                V100::ClearLineRight,
            ).as_bytes())
            .expect("Failed to write");

        stdout.flush().expect("failed to flush");

        index
    };

    app_state
        .protocol_state
        .broadcast_data(message.as_bytes().to_vec())
        .await
        .expect(&format!("Failed to broadcast message #{}", index));
}
