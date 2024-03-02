use std::io::{stdout, Write};
use std::net::SocketAddr;
use crate::protocol::frames::ProtocolMessage;
use crate::types::package::AlertPackageLevel;
use crate::types::state::AppState;
use crate::types::ui::V100;
use crate::utils::ui::UITerminal;

pub fn send_message(
    app_state: AppState,
    ui: UITerminal,
    message: &str,
    addr: SocketAddr,
) {
    let index = {
        let mut stdout = stdout().lock();
        stdout
            .write(format!(
                "{}",
                V100::GoLineUp(1),
            ).as_bytes())
            .expect("Failed to write");

        let index = ui.new_message("YOU", message);

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

    let mut lock = app_state.write_lock().expect("---Failed to acquire write lock");

    let (ref mut stream, _) = lock.streams.get_mut(&addr).expect("Should have target address saved");

    if let Err(e) = ProtocolMessage::Data(message.as_bytes().to_vec())
        .send_to_stream(stream) {
        ui.new_message(
            &format!("System: {}", AlertPackageLevel::ERROR),
            &format!("Failed to send message #{} - {}", index, e),
        );
    }
}
