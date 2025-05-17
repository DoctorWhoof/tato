use super::*;
use core::ops::{Index, IndexMut};

/// Unique identifier for a tile. Starts at zero when chip is reset.
#[derive(Debug, Clone, Copy, Eq, PartialOrd, Ord, PartialEq, Hash, Default)]
pub struct TileID(pub u16);

/// An array of clusters, each holding 8 pixels
#[derive(Debug, Clone, Hash, PartialEq, Default)]
pub struct Tile<const BITS_PER_PIXEL: usize> {
    pub clusters: [Cluster<BITS_PER_PIXEL>; TILE_CLUSTER_COUNT],
}

impl<const BITS_PER_PIXEL: usize> Tile<BITS_PER_PIXEL> {
    pub fn get_pixel(&self, x: u8, y: u8) -> u8 {
        self.clusters[y as usize].get_subpixel(x)
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, value: u8) {
        self.clusters[y as usize].set_subpixel(x, value);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Default)]
pub struct TileEntry {
    pub id: TileID,
    pub flags: TileFlags,
    pub custom_data: u8, // Unused for now
}


// For immutable indexing
impl<T> Index<TileID> for [T] {
    type Output = T;
    fn index(&self, index: TileID) -> &Self::Output {
        &self[index.0 as usize]
    }
}

// For mutable indexing
impl<T> IndexMut<TileID> for [T] {
    fn index_mut(&mut self, index: TileID) -> &mut Self::Output {
        &mut self[index.0 as usize]
    }
}
