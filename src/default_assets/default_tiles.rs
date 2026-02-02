// Auto-generated code. Do not edit manually!
#![allow(unused)]
use crate::prelude::*;

pub const BANK_DEFAULT: Bank = Bank { colors: COLORS_DEFAULT, tiles: TILES_DEFAULT };

pub const COLORS_DEFAULT: ColorBank = ColorBank::new_from(&[
    RGBA12::with_transparency(0, 0, 0, 0),
    RGBA12::with_transparency(0, 0, 0, 7),
    RGBA12::with_transparency(3, 3, 3, 7),
    RGBA12::with_transparency(7, 7, 7, 7),
]);

pub const TILES_DEFAULT: TileBank = TileBank::new_from(&[
    Tile::<2>::new(0x0000000000000000, 0x0000000000000000),
    Tile::<2>::new(0x0055005500550055, 0xAAFFAAFFAAFFAAFD),
    Tile::<2>::new(0x0001000000000000, 0x0000000000014005),
    Tile::<2>::new(0x0100054015505554, 0x0540054005400540),
    Tile::<2>::new(0x0AA02AA8AEBAAEBA, 0xAAAAA55A29680AA0),
]);

pub const TILE_EMPTY: Cell = Cell::new(0, 0, 0);
pub const TILE_CHECKERS: Cell = Cell::new(1, 0, 291);
pub const TILE_SOLID: Cell = Cell::new(0, 0, 4096);
pub const TILE_CROSSHAIRS: Cell = Cell::new(2, 0, 256);
pub const TILE_ARROW: Cell = Cell::new(3, 0, 512);
pub const TILE_SMILEY: Cell = Cell::new(4, 0, 291);
