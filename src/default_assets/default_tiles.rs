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
    Tile::new(0x0055005500550055, 0xAAFFAAFFAAFFAAFD),
    Tile::new(0x0001000000000000, 0x0000000000014005),
    Tile::new(0x0100054015505554, 0x0540054005400540),
    Tile::new(0x055015545D755D75, 0x55555AA516940550),
];
