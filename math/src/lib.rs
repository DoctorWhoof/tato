#![no_std]

pub mod num;
pub use num::Num;

pub mod rect;
pub use rect::Rect;

pub mod prelude {
    pub use crate::num::*;
    pub use crate::rect::*;
}
