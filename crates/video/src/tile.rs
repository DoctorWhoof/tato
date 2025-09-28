use super::*;
use core::ops::{Index, IndexMut};

/// Unique identifier for a tile. Starts at zero when chip is reset.
#[derive(Debug, Clone, Copy, Eq, PartialOrd, Ord, PartialEq, Hash, Default)]
pub struct TileID(pub u8);

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

impl Tile<2> {
    /// Creates a tile from two u64 values containing packed cluster data
    /// Each u64 contains 4 clusters (8 bytes), each cluster is 2 bytes
    pub const fn new(data0: u64, data1: u64) -> Self {
        Self {
            clusters: [
                // First u64 - clusters 0-3
                Cluster { data: [(data0 >> 56) as u8, (data0 >> 48) as u8] },
                Cluster { data: [(data0 >> 40) as u8, (data0 >> 32) as u8] },
                Cluster { data: [(data0 >> 24) as u8, (data0 >> 16) as u8] },
                Cluster { data: [(data0 >> 8) as u8, data0 as u8] },
                // Second u64 - clusters 4-7
                Cluster { data: [(data1 >> 56) as u8, (data1 >> 48) as u8] },
                Cluster { data: [(data1 >> 40) as u8, (data1 >> 32) as u8] },
                Cluster { data: [(data1 >> 24) as u8, (data1 >> 16) as u8] },
                Cluster { data: [(data1 >> 8) as u8, data1 as u8] },
            ],
        }
    }
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
