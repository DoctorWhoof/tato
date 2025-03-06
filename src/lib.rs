#![no_std]
//! A tiny (4 bytes) struct that contains the state of a "virtual gamepad",
//! allowing a game to abstract the player input from the windowing back-end.
//! It does not perform polling or querying the device, it simply stores the
//! current and previous state.

/// The maint Gamepad struct. The current state needs to be populated by the
/// windowing back end (SDL2, Macroquad, Winit, etc.).

mod button;
pub use button::*;

mod dpad;
pub use dpad::*;
