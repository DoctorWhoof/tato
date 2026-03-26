use super::*;
use core::ops::{Index, IndexMut};

/// Unique identifier for a tile. Starts at zero when chip is reset.
#[derive(Debug, Clone, Copy, Eq, PartialOrd, Ord, PartialEq, Hash, Default)]
pub struct TileID(pub u8);

/// An array of clusters, each holding 8 pixels
#[derive(Debug, Clone, Copy, Hash, PartialEq, Default)]
pub struct Tile<const BITS_PER_PIXEL: usize> {
    pub clusters: [Cluster<BITS_PER_PIXEL>; TILE_CLUSTER_COUNT],
}

impl<const BITS_PER_PIXEL: usize> Tile<BITS_PER_PIXEL> {
    pub fn get_pixel(&self, x: u8, y: u8) -> u8 {
        self.clusters[y as usize].get_subpixel(x)
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, value: u8) {
        self.clusters[y as usize].set_subpixel(value, x);
    }

    /// Scroll tile pixels with wrap-around (positive = right/down)
    pub fn scroll(&mut self, delta_x: i8, delta_y: i8) {
        self.scroll_y(delta_y);
        self.scroll_x(delta_x);
    }

    /// Scroll columns right with wrap-around (positive = right)
    pub fn scroll_x(&mut self, delta_x: i8) {
        // Pre-calculate common values
        let dx = delta_x.rem_euclid(TILE_SIZE as i8) as u8;
        let total_bits = (BITS_PER_PIXEL * 8) as u32;
        let mask = u64::MAX >> (64 - total_bits);
        // See explanation for l and r shifts in scroll_x_cluster()!
        let rshift = dx as u32 * BITS_PER_PIXEL as u32;
        let lshift = total_bits - rshift;
        // Perform scroll per cluster
        for cluster in &mut self.clusters {
            // Arguments passed are pre-computed values, so that
            // we don't calculate them per-cluster!
            Self::scroll_x_cluster(cluster, rshift, lshift, mask);
        }
    }

    /// Scroll rows down with wrap-around (positive = down)
    pub fn scroll_y(&mut self, delta_y: i8) {
        let dy = delta_y.rem_euclid(TILE_SIZE as i8) as usize;
        // Using this neat little Rust slice function!
        self.clusters.rotate_right(dy);
    }

    /// Rotates the pixels of a single cluster row right by `delta_x`.
    /// Works for any BPP up to 8.
    fn scroll_x_cluster(
        cluster: &mut Cluster<BITS_PER_PIXEL>,
        rshift: u32,
        lshift: u32,
        mask: u64,
    ) {
        // Pack cluster bytes into a u64 (big-endian)
        let mut data: u64 = 0;
        for &byte in &cluster.data {
            // Pack cluster's array bytes into u64 value,
            // with zeroes on the left side
            data = (data << 8) | byte as u64;
        }
        // Rotate right within the cluster's bit width
        // That's why we need l and r shifts, the bit width doesn't always line up
        // with the 64 bit data, without them we'd scroll "into" unwanted zero values.
        let rotated = ((data >> rshift) | (data << lshift)) & mask;
        // Unpack back to bytes (big-endian)
        for i in 0..BITS_PER_PIXEL {
            cluster.data[i] = (rotated >> ((BITS_PER_PIXEL - 1 - i) * 8)) as u8;
        }
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

impl Tile<4> {
    /// Creates a tile from four u64 values containing packed cluster data
    /// Each u64 contains 2 clusters (8 bytes), each cluster is 4 bytes
    pub const fn new(data0: u64, data1: u64, data2: u64, data3: u64) -> Self {
        Self {
            clusters: [
                // First u64 - clusters 0-1 (rows 0-1)
                Cluster {
                    data: [
                        (data0 >> 56) as u8,
                        (data0 >> 48) as u8,
                        (data0 >> 40) as u8,
                        (data0 >> 32) as u8,
                    ],
                },
                Cluster {
                    data: [
                        (data0 >> 24) as u8,
                        (data0 >> 16) as u8,
                        (data0 >> 8) as u8,
                        data0 as u8,
                    ],
                },
                // Second u64 - clusters 2-3 (rows 2-3)
                Cluster {
                    data: [
                        (data1 >> 56) as u8,
                        (data1 >> 48) as u8,
                        (data1 >> 40) as u8,
                        (data1 >> 32) as u8,
                    ],
                },
                Cluster {
                    data: [
                        (data1 >> 24) as u8,
                        (data1 >> 16) as u8,
                        (data1 >> 8) as u8,
                        data1 as u8,
                    ],
                },
                // Third u64 - clusters 4-5 (rows 4-5)
                Cluster {
                    data: [
                        (data2 >> 56) as u8,
                        (data2 >> 48) as u8,
                        (data2 >> 40) as u8,
                        (data2 >> 32) as u8,
                    ],
                },
                Cluster {
                    data: [
                        (data2 >> 24) as u8,
                        (data2 >> 16) as u8,
                        (data2 >> 8) as u8,
                        data2 as u8,
                    ],
                },
                // Fourth u64 - clusters 6-7 (rows 6-7)
                Cluster {
                    data: [
                        (data3 >> 56) as u8,
                        (data3 >> 48) as u8,
                        (data3 >> 40) as u8,
                        (data3 >> 32) as u8,
                    ],
                },
                Cluster {
                    data: [
                        (data3 >> 24) as u8,
                        (data3 >> 16) as u8,
                        (data3 >> 8) as u8,
                        data3 as u8,
                    ],
                },
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
