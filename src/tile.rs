use super::*;

/// Unique identifier for a tile. Starts at zero when chip is reset.
#[derive(Debug, Clone, Copy, Eq, PartialOrd, Ord, PartialEq, Hash, Default)]
pub struct TileID(pub u16);

#[derive(Debug, Clone, Hash, PartialEq, Default)]
pub struct Tile {
    pub clusters: [Cluster<2>; TILE_CLUSTER_COUNT],
}

impl Tile {
    pub fn get_pixel(&self, x: u8, y: u8) -> u8 {
        self.clusters[y as usize].get_subpixel(x)
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, value: u8) {
        self.clusters[y as usize].set_subpixel(x, value);
    }
}
