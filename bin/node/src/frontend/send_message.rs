use std::io::{stdout, Write};
use crate::protocol::frames::ProtocolMessage;
use crate::types::state::AppState;
use crate::types::ui::V100;
use crate::utils::ui::UITerminal;

pub fn send_message(
    app_state: AppState,
    ui: UITerminal,
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

    // todo: create some wrapper so that Application does not have to know about Protocol stuff.
    let lock = &mut *app_state.write_lock().expect("---Failed to acquire write lock");
    let streams = &mut lock.streams;
    let state = &mut lock.state;

    let id = state.next();
    let data = message.as_bytes().to_vec();

    for (_, (ref mut stream, _)) in streams.iter_mut() {
        AppState::send_message(
            state,
            stream,
            ProtocolMessage::Data(id, data.clone()),
        )
            .expect(&format!("Failed to send message #{}", index));
    }
}
