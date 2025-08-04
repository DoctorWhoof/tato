// Auto-generated code. Do not edit manually!
use tato::prelude::*;

pub const PATCH_TILESET: TilesetData = TilesetData {
    tiles: &PATCH_TILES,
    colors: Some(&PATCH_COLORS),
    sub_palettes: Some(&[&PATCH_SUBPALETTE_0, &PATCH_SUBPALETTE_1]),
};

pub const PATCH_COLORS: [RGBA12; 4] = [
    RGBA12::new(0, 0, 0, 0),
    RGBA12::new(4, 4, 5, 7),
    RGBA12::new(2, 2, 2, 7),
    RGBA12::new(0, 0, 1, 7),
];

pub const PATCH_SUBPALETTE_0: [u8; 4] = [0, 1, 2, 0];

pub const PATCH_SUBPALETTE_1: [u8; 4] = [0, 1, 2, 3];

pub const PATCH_MAP: Tilemap<9> = Tilemap {
    columns: 3,
    rows: 3,
    cells: [
        Cell { id: TileID(0), flags: TileFlags(0), group: 0 },
        Cell { id: TileID(1), flags: TileFlags(0), group: 0 },
        Cell { id: TileID(2), flags: TileFlags(1), group: 0 },
        Cell { id: TileID(1), flags: TileFlags(32), group: 0 },
        Cell { id: TileID(3), flags: TileFlags(0), group: 0 },
        Cell { id: TileID(4), flags: TileFlags(1), group: 0 },
        Cell { id: TileID(2), flags: TileFlags(97), group: 0 },
        Cell { id: TileID(4), flags: TileFlags(97), group: 0 },
        Cell { id: TileID(5), flags: TileFlags(1), group: 0 },
    ],
};

pub const PATCH_TILES: [Tile<2>; 6] = [
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
];
