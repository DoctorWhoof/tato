// Auto-generated code. Do not edit manually!
use tato::prelude::*;

mod patch;

pub use patch::*;

pub const BANK_PATCH: Bank = Bank { colors: COLORS_PATCH, tiles: TILES_PATCH };

pub const COLORS_PATCH: ColorBank = ColorBank::new_from(
    &[
        RGBA12::with_transparency(0, 0, 0, 0),
        RGBA12::with_transparency(4, 5, 5, 7),
        RGBA12::with_transparency(3, 3, 3, 7),
        RGBA12::with_transparency(1, 1, 1, 7),
    ],
    &[[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]],
);

pub const TILES_PATCH: TileBank = TileBank::new_from(&[
    Tile::new(0x0000000000000000, 0x0000111100012222, 0x0012222200122222, 0x0012222200122222),
    Tile::new(0x0000000000000000, 0x1111111122222222, 0x2222222222222222, 0x2222222222222222),
    Tile::new(0x0000000000000000, 0x1111000022221000, 0x2222210022222130, 0x2222213322222133),
    Tile::new(0x2222222222222222, 0x2222222222222222, 0x2222222222222222, 0x2222222222222222),
    Tile::new(0x2222213322222133, 0x2222213322222133, 0x2222133311113333, 0x3333333033333300),
]);
