use std::io::{stdout, Write};
use crate::types::ui::V100;

pub struct UiNoHistory {
}

impl UiNoHistory {
    pub fn new() -> Self {
        print!(
            "{}{}---\n>",
            V100::ClearScreen,
            // V100::GoUpperLeft,
            V100::GoLineUp(1),
        );
        stdout().flush().expect("failed to flash stdout");
        Self {
        }
    }

    pub fn system_message(&self, msg: &str) {
        self.new_message("SYSTEM", msg)
    }

    pub fn new_message(
        &self,
        from: &str,
        msg: &str,
    ) {
        print!(
            "{}{}\r[User: {}] {}{}{}",
            V100::SaveCursorPosition,
            V100::GoLineUp(2),
            from,
            msg,
            V100::ClearLineRight,
            V100::RestoreCursorPosition,
        );
        stdout().flush().expect("failed to flash stdout");
    }
}
