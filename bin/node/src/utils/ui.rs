use std::collections::HashMap;
use std::io::{Read, stdin, stdout, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use crate::types::ui::V100;

pub struct UiNoHistory {
    last_id: AtomicUsize,
    last_index: AtomicUsize,
    msg_ids: RwLock<HashMap<usize, usize>>,
}

impl UiNoHistory {
    pub fn new() -> Self {
        print!(
            "{}{}---\n>",
            V100::ClearScreen,
            V100::GoLineUp(1),
        );
        stdout().flush().expect("failed to flash stdout");
        Self {
            last_id: AtomicUsize::new(0),
            last_index: AtomicUsize::new(0),
            msg_ids: RwLock::new(HashMap::new()),
        }
    }

    pub fn system_message(&self, msg: &str) {
        self.new_message(false, "SYSTEM", msg)
    }

    pub fn new_message(
        &self,
        save_index: bool,
        from: &str,
        msg: &str,
    ) {
        let id = self.last_id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| {
                Some(if id == usize::MAX {
                    0
                } else {
                    id + 1
                })
            })
            .expect("---failed to update last_id");
        let index = self.last_id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| {
                Some(if id == usize::MAX {
                    0
                } else {
                    id + 1
                })
            })
            .expect("---failed to update last_index");

        if save_index {
            let mut lock = self.msg_ids.write().expect("---failed to get lock on msg_ids");
            lock.insert(id, index);
        }

        print!(
            "{}{}{}{}\r[User: {}] {}{}{}",
            V100::SaveCursorPosition,
            V100::MoveWindowUp,
            V100::GoLineUp(2),
            V100::InsertBlankLines(1),
            from,
            msg,
            V100::GoLineDown(2),
            V100::RestoreCursorPosition,
        );
        stdout().flush().expect("failed to flash stdout");
    }
}
