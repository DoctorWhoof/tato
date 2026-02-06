// Auto-generated code. Do not edit manually!
use tato::prelude::*;

mod button_tower;
mod cave;

pub use button_tower::*;
pub use cave::*;

pub const BANK_CAVE: Bank = Bank { colors: COLORS_CAVE, tiles: TILES_CAVE };

pub const COLORS_CAVE: ColorBank = ColorBank::new_from(&[
    RGBA12::with_transparency(0, 0, 0, 0),
    RGBA12::with_transparency(0, 0, 0, 7),
    RGBA12::with_transparency(7, 7, 7, 7),
    RGBA12::with_transparency(4, 4, 4, 7),
    RGBA12::with_transparency(1, 1, 3, 7),
    RGBA12::with_transparency(1, 2, 4, 7),
    RGBA12::with_transparency(1, 1, 1, 7),
    RGBA12::with_transparency(0, 3, 4, 7),
    RGBA12::with_transparency(2, 2, 2, 7),
    RGBA12::with_transparency(3, 3, 3, 7),
    RGBA12::with_transparency(5, 5, 5, 7),
    RGBA12::with_transparency(0, 5, 7, 7),
    RGBA12::with_transparency(6, 6, 0, 7),
    RGBA12::with_transparency(0, 2, 2, 7),
    RGBA12::with_transparency(2, 2, 3, 7),
    RGBA12::with_transparency(0, 1, 1, 7),
]);

pub const TILES_CAVE: TileBank = TileBank::new_from(&[
    Tile::<2>::new(0x0000000000000000, 0x0000000000000000),
    Tile::<2>::new(0x5154445000001400, 0x0410400050005000),
    Tile::<2>::new(0x5005410105100004, 0x4010000000000000),
    Tile::<2>::new(0x0000000000000000, 0x0001004000010005),
    Tile::<2>::new(0x0000000980A6AAA6, 0x9A9A6A96AA69A9AA),
    Tile::<2>::new(0xAA9AAAA69AA565A9, 0x5959155501505014),
    Tile::<2>::new(0x0000000000000000, 0x5001551555555555),
    Tile::<2>::new(0x0000000000002F80, 0xFFE7FBA6BA996A75),
    Tile::<2>::new(0x0000000000000001, 0x0055055515EA1FEA),
    Tile::<2>::new(0x0000000000001111, 0x5555666559996665),
    Tile::<2>::new(0x0000000000000000, 0x0000000050015515),
    Tile::<2>::new(0x04A8006542510105, 0x1420421029415505),
    Tile::<2>::new(0x017F07EA1EAA7AAA, 0x7AAA7EAA7BEA7ABF),
    Tile::<2>::new(0x0000055415641F54, 0x15641F5415500000),
    Tile::<2>::new(0x0000055510001140, 0x1000100005550000),
    Tile::<2>::new(0x0000000820269892, 0x4841054001800240),
    Tile::<2>::new(0x5500554145150104, 0x0440100000000000),
    Tile::<2>::new(0x1A95069531553815, 0x3A803A953A953A95),
    Tile::<2>::new(0x0444111004441110, 0x0444111004441110),
    Tile::<2>::new(0x55555AAA65566656, 0x6656655665546550),
    Tile::<2>::new(0x000000FF03AA0EFF, 0x3BFF3BFF3BFD3BF5),
    Tile::<2>::new(0x3BF5EFFFAAAAFFFF, 0xFFFFFFFF55555555),
    Tile::<2>::new(0x0960F55BA96AFF6F, 0xFDBFFFFF55550000),
    Tile::<2>::new(0x3295881522400815, 0x0240001500000000),
    Tile::<2>::new(0x5999666559995555, 0x0000111100000000),
    Tile::<2>::new(0x0000055010041104, 0x1004100405500000),
    Tile::<2>::new(0x3BF53BF53BF53BF5, 0x3BF53BF53BF53BF5),
    Tile::<2>::new(0x5555555555555555, 0x165559951B952EE5),
    Tile::<2>::new(0x100084AA91A5A495, 0x598506A001AA0055),
    Tile::<2>::new(0x00FCAAFE5AFE56FA, 0x52E50A90AA4C553C),
    Tile::<2>::new(0xAAAAAAA8557C557C, 0x55F0FFC000000000),
    Tile::<2>::new(0x555556955EB55EB5, 0x1691454411110000),
    Tile::<2>::new(0x5555555554544545, 0x1111444410100000),
    Tile::<2>::new(0x3BF07BF1EBFB3AAC, 0x0FF0400155555555),
    Tile::<2>::new(0x0000A000580A5555, 0x5A95A666AA6AA9AA),
    Tile::<2>::new(0x3BF43BF4EBFB7AAD, 0x5FF5155400000000),
    Tile::<2>::new(0x4000400044045110, 0x5544155501405010),
    Tile::<2>::new(0x0000000004041110, 0x4444555540401010),
]);
