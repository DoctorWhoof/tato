#![no_std]

pub mod prelude {
    pub use crate::color::*;
    pub use crate::data::*;
    pub use crate::*;
}

pub mod color;
use color::*;
pub use color::{Color9Bit, ColorRGB24};

mod data;

mod error;

mod bg;
pub use bg::*;

mod iter;
pub use iter::*;

mod cluster;
pub use cluster::*;

mod tile;
pub use tile::*;

mod videochip;
pub use videochip::*;

// -------------------------------- Constants --------------------------------

/// FG Draw buffer height.
pub const LINE_COUNT: usize = 196;

/// All tile dimensions must be a multiple of TILE_SIZE.
pub const TILE_SIZE: u8 = 8;

// Colors and bit depth
/// Number of colors per tile (2 bits per pixel)
pub const COLORS_PER_TILE: u8 = 4;

/// Number of colors per palette (applies to FG and BG palette, 32 colors total)
pub const COLORS_PER_PALETTE: u8 = 16;

/// How many "local" palettes
/// (palettes of 4 colors that map each index to the main FG and BG palettes)
pub const LOCAL_PALETTE_COUNT: u8 = 16;

/// 4 pixels per byte (4 colors per pixel)
pub const SUBPIXELS_TILE: u8 = Cluster::<2>::PIXELS_PER_BYTE as u8;

/// 2 pixels per byte (16 colors per pixel)
pub const SUBPIXELS_FRAMEBUFFER: u8 = Cluster::<4>::PIXELS_PER_BYTE as u8;

/// Number of columns in BG Map
pub const BG_COLUMNS: u8 = 64;

/// Number of rows in BG Map
pub const BG_ROWS: u8 = 64;

/// Maximum number of columns in BG Map times tile size, in pixels.
pub const BG_WIDTH: u16 = BG_COLUMNS as u16 * TILE_SIZE as u16;

/// Maximum number of rows in BG Map times tile size, in pixels.
pub const BG_HEIGHT: u16 = BG_ROWS as u16 * TILE_SIZE as u16;

/// Maximum sprite storage length (16 Kb with Cluster<2> used).
const TILE_MEM_LEN: usize = 8182;
