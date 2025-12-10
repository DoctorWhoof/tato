// Auto-generated code. Do not edit manually!
use tato::prelude::*;

mod smileys_map;

pub use smileys_map::*;

pub const SMILEYS_TILESET: TilesetData = TilesetData {
    tiles: Some(&SMILEYS_TILES),
    colors: Some(&SMILEYS_COLORS),
    color_mappings: Some(&SMILEYS_COLOR_MAPPINGS),
};

pub const SMILEYS_COLORS: [RGBA12; 16] = [
    RGBA12::with_transparency(0, 0, 0, 0),
    RGBA12::with_transparency(4, 5, 5, 7),
    RGBA12::with_transparency(3, 3, 3, 7),
    RGBA12::with_transparency(1, 1, 1, 7),
    RGBA12::with_transparency(7, 7, 7, 7),
    RGBA12::with_transparency(4, 4, 4, 7),
    RGBA12::with_transparency(7, 6, 3, 7),
    RGBA12::with_transparency(0, 2, 1, 7),
    RGBA12::with_transparency(0, 1, 3, 7),
    RGBA12::with_transparency(0, 0, 0, 7),
    RGBA12::with_transparency(6, 4, 1, 7),
    RGBA12::with_transparency(2, 4, 2, 7),
    RGBA12::with_transparency(1, 2, 6, 7),
    RGBA12::with_transparency(6, 3, 6, 7),
    RGBA12::with_transparency(4, 6, 3, 7),
    RGBA12::with_transparency(4, 6, 7, 7),
];

pub const SMILEYS_TILES: [Tile<4>; 5] = [
    Tile::new(0x4444444444444444, 0x4444444444444445, 0x4444455544445555, 0x4444555544455555),
    Tile::new(0x4444444444444444, 0x4444444455555555, 0x5555555555555555, 0x5555555555555555),
    Tile::new(0x5555555555555566, 0x5555666655566666, 0x5566666655666966, 0x5666696656666966),
    Tile::new(0x5555555555555555, 0x5555555555555555, 0x5555555555555555, 0x5555555555555555),
    Tile::new(0x5666666656669999, 0x5566999955666999, 0x5556669955556666, 0x5555556655555555),
];

// Color mappings for tile reuse with different colors
pub const SMILEYS_COLOR_MAPPINGS: [[u8; 16]; 9] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], // #0
    [0, 1, 2, 3, 4, 5, 7, 7, 8, 9, 10, 11, 12, 13, 14, 15], // #1
    [0, 1, 2, 3, 4, 5, 8, 7, 8, 9, 10, 11, 12, 13, 14, 15], // #2
    [0, 1, 2, 3, 4, 5, 10, 7, 8, 9, 10, 11, 12, 13, 14, 15], // #3
    [0, 1, 2, 3, 4, 5, 11, 7, 8, 9, 10, 11, 12, 13, 14, 15], // #4
    [0, 1, 2, 3, 4, 5, 12, 7, 8, 9, 10, 11, 12, 13, 14, 15], // #5
    [0, 1, 2, 3, 4, 5, 13, 7, 8, 9, 10, 11, 12, 13, 14, 15], // #6
    [0, 1, 2, 3, 4, 5, 14, 7, 8, 9, 10, 11, 12, 13, 14, 15], // #7
    [0, 1, 2, 3, 4, 5, 15, 7, 8, 9, 10, 11, 12, 13, 14, 15], // #8
];
