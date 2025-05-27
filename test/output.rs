// Auto-generated code - do not edit manually
use tato::prelude::*;

pub const PALETTE_FG_PALETTE: [Color12Bit; 5] = [
    Color12Bit::new(0, 0, 0, 0),
    Color12Bit::new(0, 0, 0, 7),
    Color12Bit::new(7, 5, 5, 7),
    Color12Bit::new(3, 3, 5, 7),
    Color12Bit::new(5, 5, 5, 7),
];

pub const PALETTE_SUBPALETTE_FG: [u8; 4] = [1, 2, 3, 4];

pub const SPY_IDLE_ANIM: Anim = Anim {
    fps: 8,
    columns_per_frame: 8,
    rows_per_frame: 8,
    data_start: 8,
    data_len: 8,
};

pub const CHARS_TILESET: [Tile<2>; 9] = [
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
