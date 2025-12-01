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
pub static SMILEYS_SUBPALETTE_0: [u8; 4] = [1, 0, 5, 2];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_1: [u8; 4] = [5, 1, 3, 4];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_2: [u8; 4] = [5, 1, 6, 7];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_3: [u8; 4] = [1, 5, 8, 9];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_4: [u8; 4] = [5, 1, 10, 11];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_TILES: [Tile<2>; 13] = [
    Tile::new(0x5555555555555554, 0x5540550055005400),
    Tile::new(0x5555555555550000, 0x0000000000000000),
    Tile::new(0x5555555555551555, 0x0155005500550015),
    Tile::new(0x5400540054005400, 0x5400540054005400),
    Tile::new(0x0000000F00FF03FF, 0x0FFF0FEF3FEF3FEF),
    Tile::new(0x0000F000FF00FFC0, 0xFFF0FBF0FBFCFBFC),
    Tile::new(0x0000000000000000, 0x0000000000000000),
    Tile::new(0x0015001500150015, 0x0015001500150015),
    Tile::new(0x3FFF3FAA0FAA0FEA, 0x03FA00FF000F0000),
    Tile::new(0xFFFCAAFCAAF0ABF0, 0xAFC0FF00F0000000),
    Tile::new(0x5400550055005540, 0x5554555555555555),
    Tile::new(0x0000000000000000, 0x0000555555555555),
    Tile::new(0x0015005500550155, 0x1555555555555555),
];
