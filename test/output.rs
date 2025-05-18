// Auto-generated code - do not edit manually
use tato::prelude::*;

pub const PALETTE_PALETTE_FG: [Color9Bit; 16] = [
    Color9Bit::new(0, 0, 0),
    Color9Bit::new(0, 0, 0),
    Color9Bit::new(7, 5, 5),
    Color9Bit::new(3, 3, 5),
    Color9Bit::new(5, 5, 5),
    Color9Bit::new(0, 0, 0),
    Color9Bit::new(0, 0, 0),
    Color9Bit::new(0, 0, 0),
    Color9Bit::new(0, 0, 0),
    Color9Bit::new(0, 0, 0),
    Color9Bit::new(0, 0, 0),
    Color9Bit::new(0, 0, 0),
    Color9Bit::new(0, 0, 0),
    Color9Bit::new(0, 0, 0),
    Color9Bit::new(0, 0, 0),
    Color9Bit::new(0, 0, 0),
];

pub const SUBPALETTE_PALETTE_FG_0: [u8; 4] = [1, 2, 3, 4];

pub const ANIM_SPY_IDLE: Anim<4, 6> = Anim {
    fps: 8,
    frames: [
        Tilemap::<6> {
            columns: 2,
            data: [
                TileEntry { id: TileID(0), flags: TileFlags(0) },
                TileEntry { id: TileID(1), flags: TileFlags(0) },
                TileEntry { id: TileID(2), flags: TileFlags(0) },
                TileEntry { id: TileID(3), flags: TileFlags(0) },
                TileEntry { id: TileID(4), flags: TileFlags(0) },
                TileEntry { id: TileID(5), flags: TileFlags(0) },
            ],
        },
        Tilemap::<6> {
            columns: 2,
            data: [
                TileEntry { id: TileID(0), flags: TileFlags(0) },
                TileEntry { id: TileID(1), flags: TileFlags(0) },
                TileEntry { id: TileID(6), flags: TileFlags(0) },
                TileEntry { id: TileID(3), flags: TileFlags(0) },
                TileEntry { id: TileID(4), flags: TileFlags(0) },
                TileEntry { id: TileID(5), flags: TileFlags(0) },
            ],
        },
        Tilemap::<6> {
            columns: 2,
            data: [
                TileEntry { id: TileID(0), flags: TileFlags(0) },
                TileEntry { id: TileID(1), flags: TileFlags(0) },
                TileEntry { id: TileID(7), flags: TileFlags(0) },
                TileEntry { id: TileID(3), flags: TileFlags(0) },
                TileEntry { id: TileID(4), flags: TileFlags(0) },
                TileEntry { id: TileID(5), flags: TileFlags(0) },
            ],
        },
        Tilemap::<6> {
            columns: 2,
            data: [
                TileEntry { id: TileID(0), flags: TileFlags(0) },
                TileEntry { id: TileID(1), flags: TileFlags(0) },
                TileEntry { id: TileID(8), flags: TileFlags(0) },
                TileEntry { id: TileID(3), flags: TileFlags(0) },
                TileEntry { id: TileID(4), flags: TileFlags(0) },
                TileEntry { id: TileID(5), flags: TileFlags(0) },
            ],
        },
    ],
};

pub const TILESET_CHARS: [Tile<2>; 9] = [
    Tile {
        clusters: [
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 21] },
            Cluster { data: [0, 85] },
            Cluster { data: [0, 85] },
            Cluster { data: [0, 85] },
            Cluster { data: [21, 85] },
            Cluster { data: [21, 85] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [64, 0] },
            Cluster { data: [85, 0] },
            Cluster { data: [85, 64] },
            Cluster { data: [85, 64] },
            Cluster { data: [85, 85] },
            Cluster { data: [85, 85] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [5, 85] },
            Cluster { data: [1, 85] },
            Cluster { data: [3, 254] },
            Cluster { data: [255, 255] },
            Cluster { data: [63, 255] },
            Cluster { data: [255, 87] },
            Cluster { data: [5, 85] },
            Cluster { data: [5, 85] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [85, 80] },
            Cluster { data: [149, 64] },
            Cluster { data: [149, 160] },
            Cluster { data: [170, 192] },
            Cluster { data: [255, 192] },
            Cluster { data: [255, 64] },
            Cluster { data: [85, 64] },
            Cluster { data: [85, 64] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [1, 81] },
            Cluster { data: [0, 69] },
            Cluster { data: [0, 21] },
            Cluster { data: [0, 21] },
            Cluster { data: [0, 21] },
            Cluster { data: [0, 21] },
            Cluster { data: [0, 21] },
            Cluster { data: [0, 21] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [81, 64] },
            Cluster { data: [85, 64] },
            Cluster { data: [85, 0] },
            Cluster { data: [84, 0] },
            Cluster { data: [84, 0] },
            Cluster { data: [84, 0] },
            Cluster { data: [85, 0] },
            Cluster { data: [64, 0] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [5, 85] },
            Cluster { data: [1, 85] },
            Cluster { data: [63, 218] },
            Cluster { data: [255, 255] },
            Cluster { data: [63, 255] },
            Cluster { data: [197, 127] },
            Cluster { data: [5, 85] },
            Cluster { data: [5, 85] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [5, 85] },
            Cluster { data: [1, 85] },
            Cluster { data: [253, 90] },
            Cluster { data: [63, 255] },
            Cluster { data: [255, 255] },
            Cluster { data: [7, 255] },
            Cluster { data: [5, 85] },
            Cluster { data: [5, 85] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [5, 85] },
            Cluster { data: [1, 85] },
            Cluster { data: [193, 126] },
            Cluster { data: [63, 255] },
            Cluster { data: [255, 255] },
            Cluster { data: [63, 247] },
            Cluster { data: [5, 85] },
            Cluster { data: [5, 85] },
        ],
    },
];
