// Auto-generated code. Do not edit manually!
use tato::prelude::*;

mod smileys;

pub use smileys::*;

pub const BANK_SMILEYS: Bank = Bank { colors: COLORS_SMILEYS, tiles: TILES_SMILEYS };

pub const COLORS_SMILEYS: ColorBank = ColorBank::new_from(&[
    RGBA12::with_transparency(0, 0, 0, 0),
    RGBA12::with_transparency(4, 5, 5, 7),
    RGBA12::with_transparency(3, 3, 3, 7),
    RGBA12::with_transparency(1, 1, 1, 7),
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
]);

pub const TILES_SMILEYS: TileBank = TileBank::new_from(&[
    Tile::<2>::new(0x0000000000000001, 0x0015005500550155),
    Tile::<2>::new(0x0000000000005555, 0x5555555555555555),
    Tile::<2>::new(0x0000000500550155, 0x0555056515651565),
    Tile::<2>::new(0x0000000000000000, 0x0000000000000000),
    Tile::<2>::new(0x155515AA05AA056A, 0x015A005500050000),
    Tile::<2>::new(0x0000000A00AA02AA, 0x0AAA0A9A2A9A2A9A),
    Tile::<2>::new(0x2AAA2A550A550A95, 0x02A500AA000A0000),
]);
