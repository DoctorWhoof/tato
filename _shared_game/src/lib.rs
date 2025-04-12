mod data;
pub use data::*;

mod random;
pub use random::*;

mod scn_scrolling;
pub use scn_scrolling::*;

mod scn_simple;
pub use scn_simple::*;

mod scn_simplest;
pub use scn_simplest::*;

use tato_video::*;

use padstate::*;

#[derive(Debug, Clone, Copy)]
pub struct AppState {
    pub pad: DPad,
    pub time: f64,
    pub elapsed: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Entity {
    pub x: f32,
    pub y: f32,
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
