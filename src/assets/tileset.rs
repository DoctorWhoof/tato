use core::array::from_fn;

use tato_video::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TilesetID(pub u8);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ColorEntry {
    /// If true, color already exists in the video chip. If false, it is a new entry.
    pub reused_color: bool,
    /// The index used by the color in the video chip
    pub index: u8,
    /// The color itself
    pub value: RGBA12,
}

#[derive(Debug, Clone, Copy)]
pub struct Tileset {
    pub bank_id: u8,
    // pub palette_id: PaletteID,
    pub tile_start: u8,
    pub colors: [RGBA12; COLORS_PER_PALETTE as usize],
    pub color_count:u8,
    pub sub_palettes: [[u8; 4]; SUBPALETTE_COUNT as usize],
    pub sub_palette_count:u8,
    pub sub_palettes_start: u8,
    pub sub_palettes_len: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct TilesetData<'a> {
    pub tiles: &'a [Tile<2>],
    pub colors: Option<&'a [RGBA12]>,
    pub sub_palettes: Option<&'a [&'a [u8; COLORS_PER_TILE as usize]]>,
    // pub maps: Option< &'a [&'a [Cell; 9]]>,
    // pub anims: Option< &'a [&'a [Cell; 9]]>,
    // pub fonts: Option< &'a [&'a [Cell; 9]]>,
}

impl Default for Tileset {
    fn default() -> Self {
        Self {
            bank_id: 0,
            tile_start: 0,
            colors: from_fn(|_| RGBA12::default()),
            sub_palettes: from_fn(|_| Default::default()),
            sub_palettes_start: 0,
            sub_palettes_len: 0,
            color_count: 0,
            sub_palette_count: 0,
        }
    }
}
