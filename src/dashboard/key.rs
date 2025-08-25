#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Key {
    None,
    Text(u8),
    Tab,
    Plus,
    Minus,
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
}
