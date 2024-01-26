use std::io::{stdout, Write};
use std::net::SocketAddr;
use crate::types::state::AppState;
use crate::types::ui::V100;
use crate::utils::ui::UI;

pub fn send_message(
    app_state: AppState,
    ui: UI,
    message: &str,
    addr: SocketAddr,
) {
    {
        let mut stdout = stdout().lock();
        stdout
            .write(format!(
                "{}",
                V100::GoLineUp(1),
            ).as_bytes())
            .expect("Failed to write");

        ui.new_message(true, "YOU", message);

        stdout
            .write(format!(
                "{}{}>",
                V100::GoLineDown(1),
                V100::ClearLineRight,
            ).as_bytes())
            .expect("Failed to write");

        stdout.flush().expect("failed to flush");
    }

    app_state
        .send_stream_message(&addr, message.as_bytes())
        .expect("Failed to send message");
}
