use std::fmt::{Display, Formatter};

pub enum V100 {
    MoveWindowUp,
    MoveWindowDown,
    GoLineUp(usize),
    GoLineDown(usize),
    ClearScreen,
    SaveCursorPosition,
    RestoreCursorPosition,
    GoUpperLeft,
    ClearLineRight,
    InsertBlankSymbols(usize),
    InsertBlankLines(usize),
    // DeleteLines(usize), // doesn't actually work (not the opposite of InsertBlankLines)
    ClearLine,
}

impl Display for V100 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            V100::MoveWindowUp => "D".to_string(),
            V100::MoveWindowDown => "M".to_string(),
            V100::GoLineUp(n) => format!("[{n}A"),
            V100::GoLineDown(n) => format!("[{n}B"),
            V100::ClearScreen => "[2J".to_string(),
            V100::SaveCursorPosition => "7".to_string(),
            V100::RestoreCursorPosition => "8".to_string(),
            V100::GoUpperLeft => "[H".to_string(),
            V100::ClearLineRight => "[K".to_string(),
            V100::InsertBlankSymbols(n) => format!("[{}@", n),
            V100::InsertBlankLines(n) => format!("[{}L", n),
            // V100::DeleteLines(n) => format!("[{}M", n),
            V100::ClearLine => "[2K".to_string(),
        };
        write!(f, "\x1b{}", str)
    }
}
