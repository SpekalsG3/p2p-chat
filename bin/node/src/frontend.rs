use std::sync::mpsc::Receiver;
use crate::types::Message;

pub fn handle_messages (rx: Receiver<Message>) {
    while let Ok(message) = rx.recv() {
        println!("---message\n'{}'", message.msg);
    }

    eprintln!("---channel hangup");
}
