#![no_std]

pub mod num;
pub use num::{Float, Integer, Num, SignedNum};

pub mod prelude {
    pub use crate::num::{Float, Integer, Num, SignedNum};
    pub use crate::rect::*;
    pub use crate::vec2::*;
}

pub use libm;

pub mod rect;
pub use rect::Rect;

pub mod vec2;
pub use vec2::Vec2;
