// Auto-generated code. Do not edit manually!
#![allow(unused)]
use crate::prelude::*;

pub const DEFAULT_TILESET: TilesetData = TilesetData { tiles: Some(&DEFAULT_TILES), colors: None };

pub const TILE_EMPTY: TileID = TileID(0);
pub const TILE_CHECKERS: TileID = TileID(1);
pub const TILE_SOLID: TileID = TileID(0);
pub const TILE_CROSSHAIRS: TileID = TileID(2);
pub const TILE_ARROW: TileID = TileID(3);
pub const TILE_SMILEY: TileID = TileID(4);

#[unsafe(link_section = "__DATA,__const")]
pub static DEFAULT_TILES: [Tile<4>; 5] = [
    Tile::new(0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000),
    Tile::new(0x0000111100001111, 0x0000111100001111, 0x2222333322223333, 0x2222333322223331),
    Tile::new(0x0000000100000000, 0x0000000000000000, 0x0000000000000000, 0x0000000110000011),
    Tile::new(0x0002000000222000, 0x0222220022222220, 0x0022200000222000, 0x0022200000222000),
    Tile::new(0x0022220002222220, 0x2232232222322322, 0x2222222222111122, 0x0221122000222200),
];
