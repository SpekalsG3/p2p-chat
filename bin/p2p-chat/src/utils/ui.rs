use std::io::{stdout, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::types::ui::V100;

pub struct UITerminal {
    last_index: AtomicUsize,
}

impl UITerminal {
    pub fn new() -> Self {
        print!(
            "{}{}---\n>",
            V100::ClearScreen,
            V100::GoLineUp(1),
        );
        stdout().flush().expect("failed to flash stdout");
        Self {
            last_index: AtomicUsize::new(0),
        }
    }

    pub fn new_message(
        &self,
        from: &str,
        msg: &str,
    ) -> usize {
        let index = self.last_index
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| {
                Some(if id == usize::MAX {
                    0
                } else {
                    id + 1
                })
            })
            .expect("---failed to update last_index");

        print!(
            "{}{}{}{}\r#{index} [{from}] {msg}{}{}",
            V100::SaveCursorPosition,
            V100::MoveWindowUp,
            V100::GoLineUp(2),
            V100::InsertBlankLines(1),
            V100::GoLineDown(2),
            V100::RestoreCursorPosition,
        );

        stdout().flush().expect("failed to flash stdout");

        index
    }
}
