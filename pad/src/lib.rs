#![no_std]
//! [DPad] is A tiny (4 bytes) struct that contains the state of a "virtual gamepad",
//! allowing a game to abstract the player input from the windowing back-end.
//! It does not perform polling or querying the device, it simply stores the
//! current and previous state.
//!
//! [APad] contains all the buttons in a DPad, plus an analogue left stick. Will add more axis later.

mod button;
pub use button::*;

mod dpad;
pub use dpad::*;

mod apad;
pub use apad::*;
