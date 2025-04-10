mod scenes;
pub use scenes::*;

mod random;
pub use random::*;

use videochip::*;

use padstate::*;

#[derive(Debug, Clone, Copy)]
pub struct AppState {
    pub pad: DPad,
    pub time: f64,
    pub elapsed: f64,
}

#[derive(Debug, Clone, Copy)]
struct Entity {
    x: f32,
    y: f32,
    tile: TileID,
    flags: TileFlags,
}

// Selects a scene to change into
#[derive(Debug, PartialEq)]
pub enum Mode {
    A,
    B,
    C,
}

// Contains the actual scene payload
#[derive(Debug)]
pub enum Scene {
    A(CameraScrolling),
    B(FixedCamera),
    C(MinimalScene),
}
