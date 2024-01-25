use std::fmt::{Display, Formatter};

pub enum V100 {
    ScrollWindowUp,
    ScrollWindowDown,
    GoLineUp(usize),
    GoLineDown(usize),
    ClearScreen,
    SaveCursorPosition,
    RestoreCursorPosition,
    GoUpperLeft,
    ClearLineRight,
}

impl Display for V100 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            V100::ScrollWindowUp => "D".to_string(),
            V100::ScrollWindowDown => "M".to_string(),
            V100::GoLineUp(n) => format!("[{n}A"),
            V100::GoLineDown(n) => format!("[{n}B"),
            V100::ClearScreen => "[2J".to_string(),
            V100::SaveCursorPosition => "7".to_string(),
            V100::RestoreCursorPosition => "8".to_string(),
            V100::GoUpperLeft => "[H".to_string(),
            V100::ClearLineRight => "[K".to_string(),
        };
        write!(f, "\x1b{}", str)
    }
}
