// Auto-generated code. Do not edit manually!
use tato::prelude::*;

mod patch;
pub use patch::*;

pub const BANK_PATCH: Bank = Bank {
    colors: COLORS_PATCH,
    tiles: TileBank::from_tiles(&TILES_PATCH),
};

pub const COLORS_PATCH: ColorBank = ColorBank::new_from(&[
    RGBA12::with_transparency(0, 0, 0, 0),
    RGBA12::with_transparency(4, 5, 5, 7),
    RGBA12::with_transparency(3, 3, 3, 7),
    RGBA12::with_transparency(1, 1, 1, 7),
]);

pub const TILES_PATCH: [Tile<2>; 5] = [
    Tile::<2>::new(0x00000000005501AA, 0x06AA06AA06AA06AA),
    Tile::<2>::new(0x000000005555AAAA, 0xAAAAAAAAAAAAAAAA),
    Tile::<2>::new(0x000000005500AA40, 0xAA90AA9CAA9FAA9F),
    Tile::<2>::new(0x0000000000000000, 0x0000000000000000),
    Tile::<2>::new(0xAA9FAA9FAA9FAA9F, 0xAA7F55FFFFFCFFF0),
];
