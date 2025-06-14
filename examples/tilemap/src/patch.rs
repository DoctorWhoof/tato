// Auto-generated code - do not edit manually
use tato::prelude::*;

pub const PATCH_TILESET: TilesetData = TilesetData {
    tiles: &PATCH_TILES,
    colors: Some(&PATCH_COLORS),
    sub_palettes: Some(&[&PATCH_SUBPALETTE_0, &PATCH_SUBPALETTE_1]),
};

pub const PATCH_COLORS: [Color12Bit; 7] = [
    Color12Bit::new(0, 0, 0, 0),
    Color12Bit::new(5, 5, 6, 7),
    Color12Bit::new(3, 3, 3, 7),
    Color12Bit::new(1, 1, 2, 7),
    Color12Bit::new(7, 7, 7, 7),
    Color12Bit::new(0, 4, 6, 7),
    Color12Bit::new(6, 6, 0, 7),
];

pub const PATCH_SUBPALETTE_0: [u8; 4] = [0, 1, 2, 3];

pub const PATCH_SUBPALETTE_1: [u8; 4] = [0, 4, 5, 6];

pub const PATCH_MAP: [Cell; 9] = [
    Cell { id: TileID(0), flags: TileFlags(0) },
    Cell { id: TileID(1), flags: TileFlags(0) },
    Cell { id: TileID(2), flags: TileFlags(0) },
    Cell { id: TileID(3), flags: TileFlags(0) },
    Cell { id: TileID(4), flags: TileFlags(0) },
    Cell { id: TileID(5), flags: TileFlags(0) },
    Cell { id: TileID(6), flags: TileFlags(0) },
    Cell { id: TileID(7), flags: TileFlags(0) },
    Cell { id: TileID(8), flags: TileFlags(0) },
];

pub const DEFAULT_TILES_MAP: [Cell; 9] = [
    Cell { id: TileID(9), flags: TileFlags(0) },
    Cell { id: TileID(10), flags: TileFlags(1) },
    Cell { id: TileID(11), flags: TileFlags(1) },
    Cell { id: TileID(12), flags: TileFlags(1) },
    Cell { id: TileID(13), flags: TileFlags(1) },
    Cell { id: TileID(14), flags: TileFlags(1) },
    Cell { id: TileID(15), flags: TileFlags(1) },
    Cell { id: TileID(16), flags: TileFlags(1) },
    Cell { id: TileID(17), flags: TileFlags(1) },
];

pub const PATCH_TILES: [Tile<2>; 18] = [
    Tile {
        clusters: [
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 85] },
            Cluster { data: [1, 170] },
            Cluster { data: [6, 170] },
            Cluster { data: [6, 170] },
            Cluster { data: [6, 170] },
            Cluster { data: [6, 170] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [85, 85] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [85, 0] },
            Cluster { data: [170, 64] },
            Cluster { data: [170, 144] },
            Cluster { data: [170, 156] },
            Cluster { data: [170, 159] },
            Cluster { data: [170, 159] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [6, 170] },
            Cluster { data: [6, 170] },
            Cluster { data: [6, 170] },
            Cluster { data: [6, 170] },
            Cluster { data: [6, 170] },
            Cluster { data: [6, 170] },
            Cluster { data: [6, 170] },
            Cluster { data: [6, 170] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [170, 159] },
            Cluster { data: [170, 159] },
            Cluster { data: [170, 159] },
            Cluster { data: [170, 159] },
            Cluster { data: [170, 159] },
            Cluster { data: [170, 159] },
            Cluster { data: [170, 159] },
            Cluster { data: [170, 159] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [6, 170] },
            Cluster { data: [6, 170] },
            Cluster { data: [6, 170] },
            Cluster { data: [6, 170] },
            Cluster { data: [1, 170] },
            Cluster { data: [0, 85] },
            Cluster { data: [0, 63] },
            Cluster { data: [0, 15] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [85, 85] },
            Cluster { data: [255, 255] },
            Cluster { data: [255, 255] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [170, 159] },
            Cluster { data: [170, 159] },
            Cluster { data: [170, 159] },
            Cluster { data: [170, 159] },
            Cluster { data: [170, 127] },
            Cluster { data: [85, 255] },
            Cluster { data: [255, 252] },
            Cluster { data: [255, 240] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [85, 85] },
            Cluster { data: [85, 85] },
            Cluster { data: [85, 85] },
            Cluster { data: [85, 85] },
            Cluster { data: [85, 85] },
            Cluster { data: [85, 85] },
            Cluster { data: [85, 85] },
            Cluster { data: [85, 85] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
            Cluster { data: [170, 170] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [85, 170] },
            Cluster { data: [85, 170] },
            Cluster { data: [85, 170] },
            Cluster { data: [85, 170] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [160, 2] },
            Cluster { data: [128, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [128, 0] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [10, 160] },
            Cluster { data: [42, 168] },
            Cluster { data: [162, 138] },
            Cluster { data: [162, 138] },
            Cluster { data: [170, 170] },
            Cluster { data: [160, 10] },
            Cluster { data: [40, 40] },
            Cluster { data: [10, 160] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [10, 168] },
            Cluster { data: [2, 168] },
            Cluster { data: [2, 168] },
            Cluster { data: [10, 168] },
            Cluster { data: [42, 136] },
            Cluster { data: [170, 0] },
            Cluster { data: [168, 0] },
            Cluster { data: [32, 0] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [2, 0] },
            Cluster { data: [10, 128] },
            Cluster { data: [42, 160] },
            Cluster { data: [170, 168] },
            Cluster { data: [10, 128] },
            Cluster { data: [10, 128] },
            Cluster { data: [10, 128] },
            Cluster { data: [10, 128] },
        ],
    },
];
