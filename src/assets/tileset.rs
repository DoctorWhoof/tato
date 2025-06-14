use tato_video::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct Tileset {
    pub bank_id: u8,
    // pub palette_id: PaletteID,
    pub tile_start: u16,
    pub tiles_count: u16,
    pub colors_start: u8,
    pub colors_len: u8,
    // pub sub_palette_start: u8,
    // pub sub_palette_len: u8,
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
