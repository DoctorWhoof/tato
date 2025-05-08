// #![no_std]

pub mod prelude {
    pub use crate::color::*;
    pub use crate::data::*;
    pub use crate::*;
}

mod bg;
pub use bg::*;

mod cluster;
pub use cluster::*;

pub mod color;
use color::*;

mod data;

mod error;

mod iter;
pub use iter::*;

mod sprite;
use sprite::*;

mod tile;
pub use tile::*;

mod tile_bank;
pub use tile_bank::*;

mod tile_flags;
pub use tile_flags::*;

mod videochip;
pub use videochip::*;

/// A callback used to modify the iterator, called once on every line at
/// an X position determined by [VideoChip::horizontal_irq_position].
/// The parameters are a mutable reference to the iterator, a read-only reference to
/// the VideoChip and a u16 value with the current line number.
pub type HorizontalIRQ = fn(&mut PixelIter, &VideoChip, u16);

// -------------------------------- Constants --------------------------------

/// Maximum number of video scanlines
pub const MAX_LINES: usize = 256;

/// Maximum sprite storage length (8 Kb with Cluster<2> used).
/// TODO: May be increased to 1024?
pub const TILE_COUNT: usize = 512;

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
pub const LOCAL_PALETTE_COUNT: u8 = 16;

/// Number of columns in BG Map
pub const BG_MAX_COLUMNS: u8 = 64;

/// Number of rows in BG Map
pub const BG_MAX_ROWS: u8 = 64;

/// Limits how many sprites can be visible in a single video scanline. Also affects
/// the memory amount used by the videochip, since more sprites per line need more buffer space.
pub const SPRITES_PER_LINE: usize = 16;

/// A "slot" is a way to divide each scanline in a way the pixel iterator can use to
/// quickly determine if any sprite is present in that section.
pub const SLOTS_PER_LINE: usize = 16;
