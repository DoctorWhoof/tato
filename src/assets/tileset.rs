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
    pub value: Color12Bit,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Tileset {
    pub bank_id: u8,
    // pub palette_id: PaletteID,
    pub tile_start: u16,
    pub tiles_count: u16,
    pub color_entries: [ColorEntry; COLORS_PER_PALETTE as usize],
    pub color_count: u8,

    // TODO: Get rid of these once color entry management is in!
    // pub colors_start: u8,
    // pub colors_len: u8,

    pub sub_palettes_start: u8,
    pub sub_palettes_len: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct TilesetData<'a> {
    pub tiles: &'a [Tile<2>],
    pub colors: Option<&'a [Color12Bit]>,
    pub sub_palettes: Option<&'a [&'a [u8; COLORS_PER_TILE as usize]]>,
    // pub maps: Option< &'a [&'a [Cell; 9]]>,
    // pub anims: Option< &'a [&'a [Cell; 9]]>,
    // pub fonts: Option< &'a [&'a [Cell; 9]]>,
}
