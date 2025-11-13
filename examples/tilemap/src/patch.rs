// Auto-generated code. Do not edit manually!
use tato::prelude::*;

mod patch_map;

pub use patch_map::*;

pub const PATCH_TILESET: TilesetData = TilesetData {
    tiles: Some(&PATCH_TILES),
    colors: Some(&PATCH_COLORS),
    sub_palettes: Some(&[&PATCH_SUBPALETTE_0, &PATCH_SUBPALETTE_1, &PATCH_SUBPALETTE_2]),
};

#[unsafe(link_section = "__DATA,__const")]
pub static PATCH_COLORS: [RGBA12; 4] = [
    RGBA12::with_transparency(0, 0, 0, 0),
    RGBA12::with_transparency(4, 4, 5, 7),
    RGBA12::with_transparency(2, 2, 2, 7),
    RGBA12::with_transparency(0, 0, 1, 7),
];

#[unsafe(link_section = "__DATA,__const")]
pub static PATCH_SUBPALETTE_0: [u8; 4] = [0, 1, 2, 0];

#[unsafe(link_section = "__DATA,__const")]
pub static PATCH_SUBPALETTE_1: [u8; 4] = [0, 1, 2, 3];

#[unsafe(link_section = "__DATA,__const")]
pub static PATCH_SUBPALETTE_2: [u8; 4] = [2, 0, 0, 0];

#[unsafe(link_section = "__DATA,__const")]
pub static PATCH_TILES: [Tile<2>; 6] = [
    Tile::new(0x00000000005501AA, 0x06AA06AA06AA06AA),
    Tile::new(0x000000005555AAAA, 0xAAAAAAAAAAAAAAAA),
    Tile::new(0x000000005500AA40, 0xAA90AA9CAA9FAA9F),
    Tile::new(0x0000000000000000, 0x0000000000000000),
    Tile::new(0xAA9FAA9FAA9FAA9F, 0xAA9FAA9FAA9FAA9F),
    Tile::new(0xAA9FAA9FAA9FAA9F, 0xAA7F55FFFFFCFFF0),
];
