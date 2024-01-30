use std::fmt::Display;
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use crate::types::ui::{ANSIColors, V100};

struct UITerminalInner {
    last_index: AtomicUsize,
}

pub struct UITerminal(Arc<UITerminalInner>);

impl UITerminal {
    pub fn new() -> Self {
        print!(
            "{}{}---\n>",
            V100::ClearScreen,
            V100::GoLineUp(1),
        );
        stdout().flush().expect("failed to flash stdout");
        Self(Arc::new(UITerminalInner {
            last_index: AtomicUsize::new(0),
        }))
    }

    pub fn new_message(
        &self,
        from: &str,
        msg: &str,
    ) -> usize {
        let index = self.0.last_index
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

impl Clone for UITerminal {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
