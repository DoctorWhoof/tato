// Auto-generated code. Do not edit manually!
use tato::prelude::*;

mod smileys_map;

pub use smileys_map::*;

pub const SMILEYS_TILESET: TilesetData = TilesetData {
    tiles: Some(&SMILEYS_TILES),
    colors: Some(&SMILEYS_COLORS),
    sub_palettes: Some(&[
        &SMILEYS_SUBPALETTE_0,
        &SMILEYS_SUBPALETTE_1,
        &SMILEYS_SUBPALETTE_2,
        &SMILEYS_SUBPALETTE_3,
        &SMILEYS_SUBPALETTE_4,
    ]),
};

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_COLORS: [RGBA12; 12] = [
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

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_0: [u8; 4] = [0, 1, 5, 2];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_1: [u8; 4] = [1, 5, 3, 4];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_2: [u8; 4] = [6, 5, 1, 7];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_3: [u8; 4] = [1, 5, 8, 9];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_4: [u8; 4] = [1, 10, 5, 11];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_TILES: [Tile<2>; 5] = [
    Tile::new(0x0000000000000001, 0x0015005500550155),
    Tile::new(0x0000000000005555, 0x5555555555555555),
    Tile::new(0x5555555F55FF57FF, 0x5FFF5FEF7FEF7FEF),
    Tile::new(0x5555555555555555, 0x5555555555555555),
    Tile::new(0x7FFF7FAA5FAA5FEA, 0x57FA55FF555F5555),
];
