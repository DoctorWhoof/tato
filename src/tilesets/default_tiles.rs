// Auto-generated code - do not edit manually
use crate::prelude::*;

pub const DEFAULT_TILESET: TilesetData = TilesetData {
    tiles: &DEFAULT_TILES,
    colors: None,
    sub_palettes: None,
};

pub const DEFAULT_COLORS: [Color12Bit; 4] = [
    Color12Bit::new(0, 0, 0, 0),
    Color12Bit::new(2, 2, 4, 7),
    Color12Bit::new(3, 4, 5, 7),
    Color12Bit::new(7, 7, 7, 7),
];

pub const DEFAULT_SUBPALETTE_0: [u8; 4] = [1, 2, 3, 0];

pub const TILE_CHECKERS: TileID = TileID(0);
pub const TILE_EMPTY: TileID = TileID(1);
pub const TILE_SOLID: TileID = TileID(2);
pub const TILE_CROSSHAIRS: TileID = TileID(3);
pub const TILE_ARROW: TileID = TileID(4);
pub const TILE_SMILEY: TileID = TileID(5);

pub const DEFAULT_TILES: [Tile<2>; 6] = [
    Tile {
        clusters: [
            Cluster { data: [0, 85] },
            Cluster { data: [0, 85] },
            Cluster { data: [0, 85] },
            Cluster { data: [0, 85] },
            Cluster { data: [170, 255] },
            Cluster { data: [170, 255] },
            Cluster { data: [170, 255] },
            Cluster { data: [170, 255] },
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
            Cluster { data: [255, 255] },
            Cluster { data: [255, 255] },
            Cluster { data: [255, 255] },
            Cluster { data: [255, 255] },
            Cluster { data: [255, 255] },
            Cluster { data: [255, 255] },
            Cluster { data: [255, 255] },
            Cluster { data: [255, 255] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [240, 3] },
            Cluster { data: [192, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [0, 0] },
            Cluster { data: [192, 0] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [3, 0] },
            Cluster { data: [15, 192] },
            Cluster { data: [63, 240] },
            Cluster { data: [255, 252] },
            Cluster { data: [15, 192] },
            Cluster { data: [15, 192] },
            Cluster { data: [15, 192] },
            Cluster { data: [15, 192] },
        ],
    },
    Tile {
        clusters: [
            Cluster { data: [15, 240] },
            Cluster { data: [63, 252] },
            Cluster { data: [247, 223] },
            Cluster { data: [247, 223] },
            Cluster { data: [255, 255] },
            Cluster { data: [250, 175] },
            Cluster { data: [62, 188] },
            Cluster { data: [15, 240] },
        ],
    },
];
