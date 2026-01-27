//! Builder types for converting images to tiles, maps, and animations.

use tato_video::TILE_SIZE;

const TILE_LEN: usize = TILE_SIZE as usize * TILE_SIZE as usize;

/// Pixel data for a single tile (64 color indices).
pub(crate) type Pixels = [u8; TILE_LEN];

mod anim;
pub(crate) use anim::*;

mod bank;
pub use bank::*;

mod group;
pub use group::*;

mod map;
pub(crate) use map::*;

mod single_tile;
pub(crate) use single_tile::*;

mod palette;
pub use palette::*;
