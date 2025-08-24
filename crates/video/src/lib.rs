#![no_std]

mod cell;
pub use cell::*;

pub mod color;
pub use color::*;

mod cluster;
pub use cluster::*;

mod error;

mod iter;
pub use iter::*;

mod sprite;
use sprite::*;

mod tile;
pub use tile::*;

mod tilemap;
pub use tilemap::*;

mod tilemap_ref;
pub use tilemap_ref::*;

mod tile_flags;
pub use tile_flags::*;

mod video_mem;
pub use video_mem::*;

mod video_chip;
pub use video_chip::*;

pub use tato_math as math;

/// A callback used to modify the iterator, called once on every new scanline.
/// The parameters are:
/// - Mutable reference to the iterator
/// - Read-only reference to the VideoChip
/// - Read only reference to the current tilemap
pub type VideoIRQ = fn(&mut PixelIter, &VideoChip, &TilemapRef);

// -------------------------------- Constants --------------------------------

/// Maximum number of video scanlines
pub const MAX_LINES: usize = 240;
pub const MAX_VERTICAL_LINES: usize = 320;

/// Maximum number of simultaneous sprites on a single frame
pub const MAX_SPRITES: usize = 256;

/// Limits how many sprites can be visible in a single video scanline. Also affects
/// the memory amount used by the videochip, since more sprites per line need more buffer space.
pub const SPRITES_PER_LINE: usize = 16;

/// A "slot" is a way to divide each scanline in a way the pixel iterator can use to
/// quickly determine if any sprite is present in that section.
pub const SLOTS_PER_LINE: usize = 16;

/// Maximum tile count in video memory (4 Kb with Cluster<2> used).
pub const TILE_COUNT: usize = 256;

/// Determines the X and Y size used by every tile.
pub const TILE_SIZE: u8 = 8;

/// The number of pixels in a tile
pub const TILE_PIXEL_COUNT: usize = TILE_SIZE as usize * TILE_SIZE as usize;

/// The number of pixel clusters in a tile.
pub const TILE_CLUSTER_COUNT: usize = TILE_PIXEL_COUNT / PIXELS_PER_CLUSTER as usize;

/// Number of colors per tile (2 bits per pixel)
pub const COLORS_PER_TILE: u8 = 4;

/// Number of colors per palette (applies to FG and BG palette, 32 colors total)
pub const COLORS_PER_PALETTE: u8 = 16;

/// How many "local" palettes
/// (palettes of 4 colors that map each index to the main FG and BG palettes)
pub const SUBPALETTE_COUNT: u8 = 32;

/// Maximum number of BG Tiles
pub const BG_LEN: usize = 1024;

pub const BG_BANK_COUNT: usize = 4;
pub const TILE_BANK_COUNT: usize = 4;
