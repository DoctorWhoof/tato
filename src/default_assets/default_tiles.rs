// Auto-generated code. Do not edit manually!
#![allow(unused)]
use crate::prelude::*;

pub const DEFAULT_TILESET: TilesetData = TilesetData {
    tiles: Some(&DEFAULT_TILES),
    colors: None,
    sub_palettes: None,
};

pub const TILE_EMPTY: TileID = TileID(0);
pub const TILE_CHECKERS: TileID = TileID(1);
pub const TILE_SOLID: TileID = TileID(0);
pub const TILE_CROSSHAIRS: TileID = TileID(2);
pub const TILE_ARROW: TileID = TileID(3);
pub const TILE_SMILEY: TileID = TileID(4);

#[unsafe(link_section = "__DATA,__const")]
pub static DEFAULT_TILES: [Tile<2>; 5] = [
    Tile::new(0x0000000000000000, 0x0000000000000000),
    Tile::new(0x00AA00AA00AA00AA, 0x55FF55FF55FF55FE),
    Tile::new(0x0002000000000000, 0x000000000002800A),
    Tile::new(0x02000A802AA0AAA8, 0x0A800A800A800A80),
    Tile::new(0x0AA02AA8AEBAAEBA, 0xAAAAA55A29680AA0),
];
