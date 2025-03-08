use crate::*;

impl Default for Frame {
    fn default() -> Self {
        Frame {
            start: Anchor::CursorStart(0),
            end: Anchor::CursorEnd(0),
            direction: Direction::Horizontal,
            margin: Some(10),
            gap: Some(5),
            children: None,
        }
    }
}

impl Frame {
    pub fn root(len: u16, direction: Direction) -> Frame {
        Frame {
            start: Anchor::CursorStart(0),
            end: Anchor::CursorEnd(len),
            direction,
            margin: None,
            gap: None,
            children: None,
        }
    }

    pub fn from_start(len: u16, direction: Direction) -> Frame {
        Frame {
            start: Anchor::CursorStart(0),
            end: Anchor::CursorEnd(len),
            direction,
            margin: None,
            gap: None,
            children: None,
        }
    }

    pub fn from_end(len: u16, direction: Direction) -> Frame {
        Frame {
            start: Anchor::CursorEnd(0),
            end: Anchor::CursorEnd(len),
            direction,
            margin: None,
            gap: None,
            children: None,
        }
    }
}
