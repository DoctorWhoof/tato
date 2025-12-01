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
        &SMILEYS_SUBPALETTE_5,
        &SMILEYS_SUBPALETTE_6,
        &SMILEYS_SUBPALETTE_7,
        &SMILEYS_SUBPALETTE_8,
        &SMILEYS_SUBPALETTE_9,
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
pub static SMILEYS_SUBPALETTE_0: [u8; 4] = [0, 1, 0, 0];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_1: [u8; 4] = [1, 2, 5, 0];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_2: [u8; 4] = [1, 3, 5, 0];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_3: [u8; 4] = [1, 4, 5, 0];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_4: [u8; 4] = [1, 5, 6, 0];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_5: [u8; 4] = [1, 5, 7, 0];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_6: [u8; 4] = [1, 5, 8, 0];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_7: [u8; 4] = [1, 5, 9, 0];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_8: [u8; 4] = [1, 5, 10, 0];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_SUBPALETTE_9: [u8; 4] = [1, 5, 11, 0];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_TILES: [Tile<2>; 7] = [
    Tile::new(0x0000000000000001, 0x0015005500550155),
    Tile::new(0x0000000000005555, 0x5555555555555555),
    Tile::new(0x0000000500550155, 0x0555056515651565),
    Tile::new(0x5555555555555555, 0x5555555555555555),
    Tile::new(0x155515AA05AA056A, 0x015A005500050000),
    Tile::new(0x0000000A00AA02AA, 0x0AAA0A9A2A9A2A9A),
    Tile::new(0x2AAA2A550A550A95, 0x02A500AA000A0000),
];
