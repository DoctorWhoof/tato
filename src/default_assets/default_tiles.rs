// Auto-generated code. Do not edit manually!
#![allow(unused)]
use crate::prelude::*;

pub const BANK_DEFAULT: Bank = Bank { colors: COLORS_DEFAULT, tiles: TILES_DEFAULT };

pub const COLORS_DEFAULT: ColorBank = ColorBank::new_from(
    &[
        RGBA12::with_transparency(0, 0, 0, 0),
        RGBA12::with_transparency(0, 0, 0, 7),
        RGBA12::with_transparency(3, 3, 3, 7),
        RGBA12::with_transparency(7, 7, 7, 7),
    ],
    &[
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        [1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    ],
);

pub const TILES_DEFAULT: TileBank = TileBank::new_from(&[
    Tile::new(0x0000000000000000, 0x0000000000000000, 0x0000000000000000, 0x0000000000000000),
    Tile::new(0x0000111100001111, 0x0000111100001111, 0x2222333322223333, 0x2222333322223331),
    Tile::new(0x0000000100000000, 0x0000000000000000, 0x0000000000000000, 0x0000000110000011),
    Tile::new(0x0002000000222000, 0x0222220022222220, 0x0022200000222000, 0x0022200000222000),
    Tile::new(0x0022220002222220, 0x2232232222322322, 0x2222222222111122, 0x0221122000222200),
]);

pub const TILE_EMPTY: Cell = Cell::new(0, 0, 0, 0);
pub const TILE_CHECKERS: Cell = Cell::new(1, 0, 0, 0);
pub const TILE_SOLID: Cell = Cell::new(0, 0, 1, 0);
pub const TILE_CROSSHAIRS: Cell = Cell::new(2, 0, 0, 0);
pub const TILE_ARROW: Cell = Cell::new(3, 0, 0, 0);
pub const TILE_SMILEY: Cell = Cell::new(4, 0, 0, 0);
