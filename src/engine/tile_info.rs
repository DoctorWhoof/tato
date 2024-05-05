use crate::TileID;

/// Used as an argument when drawing a tile.
pub struct TileInfo {
    pub tile: TileID,
    pub flip_h: bool,
    pub flip_v: bool,
    pub depth:u8
}