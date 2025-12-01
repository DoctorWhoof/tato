// Auto-generated code. Do not edit manually!
use tato::prelude::*;

mod patch_map;

pub use patch_map::*;

pub const PATCH_TILESET: TilesetData = TilesetData {
    tiles: Some(&PATCH_TILES),
    colors: Some(&PATCH_COLORS),
    sub_palettes: Some(&[&PATCH_SUBPALETTE_0]),
};

#[unsafe(link_section = "__DATA,__const")]
pub static PATCH_COLORS: [RGBA12; 4] = [
    RGBA12::with_transparency(0, 0, 0, 0),
    RGBA12::with_transparency(4, 4, 5, 7),
    RGBA12::with_transparency(2, 2, 2, 7),
    RGBA12::with_transparency(0, 0, 1, 7),
];

#[unsafe(link_section = "__DATA,__const")]
pub static PATCH_SUBPALETTE_0: [u8; 4] = [0, 2, 1, 3];

#[unsafe(link_section = "__DATA,__const")]
pub static PATCH_TILES: [Tile<2>; 5] = [
    Tile::new(0x0000000000AA0255, 0x0955095509550955),
    Tile::new(0x00000000AAAA5555, 0x5555555555555555),
    Tile::new(0x00000000AA005580, 0x5560556C556F556F),
    Tile::new(0x5555555555555555, 0x5555555555555555),
    Tile::new(0x556F556F556F556F, 0x55BFAAFFFFFCFFF0),
];
