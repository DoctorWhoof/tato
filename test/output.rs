// Auto-generated code - do not edit manually
use tato::prelude::*;

pub const PALETTE_FG_TILESET: TilesetData = TilesetData {
    tiles: &CHARS_TILES,
    colors: Some(&CHARS_COLORS),
    sub_palettes: None,
};

pub const PALETTE_FG_COLORS: [Color12Bit; 5] = [
    Color12Bit::new(0, 0, 0, 0),
    Color12Bit::new(0, 0, 0, 7),
    Color12Bit::new(7, 5, 5, 7),
    Color12Bit::new(3, 3, 5, 7),
    Color12Bit::new(5, 5, 5, 7),
];

pub const SPY_IDLE_ANIM: Anim = Anim {
    fps: 8,
    columns_per_frame: 8,
    rows_per_frame: 8,
    data_start: 8,
    data_len: 8,
};

pub const CHARS_TILES: [Tile<2>; 0] = [];
