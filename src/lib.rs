use std::collections::HashMap;

// #![no_std]
// extern crate alloc;
// use alloc::vec::Vec;
use normal_float::UNormal16;

// mod frame;
mod layout;
pub use layout::Layout;


pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

pub enum Direction {
    Horizontal,
    Vertical,
}

// pub struct Frame {
//     pub start: Anchor,
//     pub end: Anchor,
//     pub direction: Direction,
//     pub margin: Option<u16>,
//     pub gap: Option<u16>,
//     pub children: Option<Vec<Frame>>,
// }

pub enum Anchor {
    CursorStart(u16), // Double cursor, this is from the start of layout
    CursorEnd(u16),   // Double cursor, this is from the end of layout
    Ratio(UNormal16), // Ratio of the "left over" space in betweeen the two cursors
}
