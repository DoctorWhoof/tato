// Auto-generated code. Do not edit manually!
#![allow(unused)]
use tato::prelude::*;

pub const ASTRO_TILESET: TilesetData = TilesetData {
    tiles: Some(&ASTRO_TILES),
    colors: Some(&ASTRO_COLORS),
    sub_palettes: Some(&[&ASTRO_SUBPALETTE_0, &ASTRO_SUBPALETTE_1]),
};

#[unsafe(link_section = "__DATA,__const")]
pub static ASTRO_COLORS: [RGBA12; 4] = [
    RGBA12::with_transparency(0, 0, 0, 0),
    RGBA12::with_transparency(7, 7, 7, 7),
    RGBA12::with_transparency(0, 0, 0, 7),
    RGBA12::with_transparency(1, 1, 6, 7),
];

#[unsafe(link_section = "__DATA,__const")]
pub static ASTRO_SUBPALETTE_0: [u8; 4] = [0, 1, 2, 0];

#[unsafe(link_section = "__DATA,__const")]
pub static ASTRO_SUBPALETTE_1: [u8; 4] = [0, 1, 2, 3];

#[unsafe(link_section = "__DATA,__const")]
pub static STRIP_ASTRO: [Tilemap<4>; 24] = [
    Tilemap {
        cells: [
            Cell::new(0, 0, 0, 0),
            Cell::new(0, 128, 0, 0),
            Cell::new(1, 0, 0, 0),
            Cell::new(1, 128, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(2, 0, 0, 0),
            Cell::new(2, 128, 0, 0),
            Cell::new(3, 0, 0, 0),
            Cell::new(3, 128, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(4, 0, 0, 0),
            Cell::new(5, 0, 0, 0),
            Cell::new(6, 0, 0, 0),
            Cell::new(6, 128, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(0, 0, 0, 0),
            Cell::new(0, 128, 0, 0),
            Cell::new(1, 0, 0, 0),
            Cell::new(1, 128, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(7, 0, 0, 0),
            Cell::new(7, 128, 0, 0),
            Cell::new(8, 0, 0, 0),
            Cell::new(9, 0, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(0, 0, 0, 0),
            Cell::new(0, 128, 0, 0),
            Cell::new(1, 0, 0, 0),
            Cell::new(1, 128, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(7, 0, 0, 0),
            Cell::new(7, 128, 0, 0),
            Cell::new(9, 128, 0, 0),
            Cell::new(8, 128, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(2, 0, 0, 0),
            Cell::new(2, 128, 0, 0),
            Cell::new(3, 0, 0, 0),
            Cell::new(3, 128, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(10, 0, 0, 0),
            Cell::new(10, 128, 0, 0),
            Cell::new(11, 0, 0, 0),
            Cell::new(12, 0, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(2, 0, 0, 0),
            Cell::new(2, 128, 0, 0),
            Cell::new(13, 0, 0, 0),
            Cell::new(13, 128, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(10, 0, 0, 0),
            Cell::new(10, 128, 0, 0),
            Cell::new(12, 128, 0, 0),
            Cell::new(11, 128, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(4, 0, 0, 0),
            Cell::new(5, 0, 0, 0),
            Cell::new(6, 0, 0, 0),
            Cell::new(6, 128, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(10, 0, 0, 0),
            Cell::new(14, 0, 0, 0),
            Cell::new(15, 0, 0, 0),
            Cell::new(16, 0, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(4, 0, 0, 0),
            Cell::new(5, 0, 0, 0),
            Cell::new(6, 0, 0, 0),
            Cell::new(6, 128, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(10, 0, 0, 0),
            Cell::new(14, 0, 0, 0),
            Cell::new(17, 0, 0, 0),
            Cell::new(18, 0, 0, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(19, 0, 0, 0),
            Cell::new(19, 128, 0, 0),
            Cell::new(20, 0, 1, 0),
            Cell::new(21, 0, 1, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(22, 0, 0, 0),
            Cell::new(22, 128, 0, 0),
            Cell::new(23, 0, 1, 0),
            Cell::new(24, 0, 1, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(19, 0, 0, 0),
            Cell::new(19, 128, 0, 0),
            Cell::new(25, 0, 1, 0),
            Cell::new(26, 0, 1, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(19, 0, 0, 0),
            Cell::new(19, 128, 0, 0),
            Cell::new(27, 0, 1, 0),
            Cell::new(28, 0, 1, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(29, 0, 0, 0),
            Cell::new(29, 128, 0, 0),
            Cell::new(30, 0, 1, 0),
            Cell::new(31, 0, 1, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(29, 0, 0, 0),
            Cell::new(29, 128, 0, 0),
            Cell::new(32, 0, 1, 0),
            Cell::new(33, 0, 1, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(22, 0, 0, 0),
            Cell::new(22, 128, 0, 0),
            Cell::new(34, 0, 1, 0),
            Cell::new(35, 0, 1, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(29, 0, 0, 0),
            Cell::new(29, 128, 0, 0),
            Cell::new(36, 0, 1, 0),
            Cell::new(37, 0, 1, 0),
        ],
        columns: 2,
        rows: 2,
    },
    Tilemap {
        cells: [
            Cell::new(19, 0, 0, 0),
            Cell::new(19, 128, 0, 0),
            Cell::new(38, 0, 1, 0),
            Cell::new(39, 0, 1, 0),
        ],
        columns: 2,
        rows: 2,
    },
];

#[unsafe(link_section = "__DATA,__const")]
pub static ASTRO_TILES: [Tile<2>; 40] = [
    Tile::new(0x0000001500550155, 0x0155015501590059),
    Tile::new(0x0015004001550145, 0x0045001500140000),
    Tile::new(0x0015005501550155, 0x0155015500550015),
    Tile::new(0x0040015501550055, 0x0015001400000000),
    Tile::new(0x0000000000150055, 0x0155015501550155),
    Tile::new(0x0000000054005500, 0x5540554056405640),
    Tile::new(0x0055000500150011, 0x0011001400050001),
    Tile::new(0x0000000000150055, 0x0155015501550159),
    Tile::new(0x0059001500400151, 0x0151004500150000),
    Tile::new(0x6500544001405500, 0x5400540054001400),
    Tile::new(0x0000001500550155, 0x0155015501550055),
    Tile::new(0x0015004001550155, 0x0055000500000000),
    Tile::new(0x5440014055005400, 0x5400540014000000),
    Tile::new(0x0140015500550015, 0x0015001400000000),
    Tile::new(0x0000540055005540, 0x5540564056405500),
    Tile::new(0x0005001500550114, 0x0141000500000000),
    Tile::new(0x5000540015005500, 0x5400554005400000),
    Tile::new(0x0005001500150004, 0x0055015500500000),
    Tile::new(0x5000440051005140, 0x0540540000000000),
    Tile::new(0x0000000000000000, 0x0000000000150055),
    Tile::new(0x0155015501550159, 0x0F5903FF003F03C3),
    Tile::new(0x5540554055406540, 0x65F0FFC0C3F0F000),
    Tile::new(0x0000000000000000, 0x0015005501550155),
    Tile::new(0x0155015501550F55, 0x03FF003F0F03000F),
    Tile::new(0x55405540554055F0, 0xFFC0FFF0FC00F000),
    Tile::new(0x0155015501550D55, 0x3FFF00FF0003003F),
    Tile::new(0x5540554056405670, 0xFFFCFFC0FCF0C000),
    Tile::new(0x015531550D550D59, 0x0F5903FF000F0000),
    Tile::new(0x5540554C557C6570, 0x65F0FFC0FC000000),
    Tile::new(0x0000000000000000, 0x0000001500550155),
    Tile::new(0x0155015501590D59, 0x0F5503FF003F0000),
    Tile::new(0x5540554065406570, 0x55F0FFC0C0000000),
    Tile::new(0x015501550D550F55, 0x03FF003F00030000),
    Tile::new(0x55405540557055F0, 0xFFC0FF00F0000000),
    Tile::new(0x01550D550D550355, 0x00FF000F00030000),
    Tile::new(0x55405570557055C0, 0xFFC0FC00F000C000),
    Tile::new(0x015501550D553F55, 0x0FFF00FF00000000),
    Tile::new(0x55405640567055C0, 0xFF00FC0000000000),
    Tile::new(0x015501553D55FF55, 0x0FFF00FF00000000),
    Tile::new(0x55405540567056FC, 0xFFC0F00000000000),
];
