// Auto-generated code. Do not edit manually!
use tato::prelude::*;

mod smileys_map;

pub use smileys_map::*;

pub const SMILEYS_TILESET: TilesetData =
    TilesetData { tiles: Some(&SMILEYS_TILES), colors: Some(&SMILEYS_COLORS) };

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_COLORS: [RGBA12; 12] = [
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
];

#[unsafe(link_section = "__DATA,__const")]
pub static SMILEYS_TILES: [Tile<2>; 5] = [
    Tile::new(0x0000000000000001, 0x0015005500550155),
    Tile::new(0x0000000000005555, 0x5555555555555555),
    Tile::new(0x5555555A55AA56AA, 0x5AAA5A9A6A9A6A9A),
    Tile::new(0x5555555555555555, 0x5555555555555555),
    Tile::new(0x6AAA6A555A555A95, 0x56A555AA555A5555),
];
