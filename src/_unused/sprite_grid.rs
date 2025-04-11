use crate::*;
use core::array::from_fn;

#[derive(Debug, Clone)]
pub(crate) struct PixelQuery {
    pub pixel: SubPixel,
    pub flags: TileFlags,
}

/// A convenient packet of data used to draw a tile as a sprite.
#[derive(Debug, Clone, Copy)]
pub struct DrawBundle {
    pub x: u8,
    pub y: u8,
    pub id: TileID,
    pub flags: TileFlags,
}

/// A convenient packet of data to be passed in some functions.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct TileBundle {
    pub x: u8, // Original x position of the sprite
    pub y: u8, // Original y position of the sprite
    pub tile: TileEntry,
    pub flags: TileFlags,
}

const CLUSTERS_PER_SCANLINE:usize = 256 / 8; // always 8 pixels per cluster

/// Const LINE_COUNT is the total number of scanlines.
#[derive(Debug, Clone)]
pub(crate) struct SpriteGrid {
    pub lines: [[PixelCluster<2>; 256 / 8]; LINE_COUNT],
}

impl SpriteGrid {
    pub fn new() -> Self {
        Self {
            lines: from_fn(|_| from_fn(|_| PixelCluster::default())),
        }
    }

    pub fn insert(&mut self, tile_clusters: &[PixelCluster<4>], bundle: TileBundle) {
        // Calculate effective sprite dimensions based on rotation
        let (width, height) = if bundle.flags.is_rotated() {
            (bundle.tile.h, bundle.tile.w) // Swap width and height for rotated sprites
        } else {
            (bundle.tile.w, bundle.tile.h)
        };

        // Calculate sprite boundaries in world coordinates
        let bundle_right = bundle.x.min(bundle.x.saturating_add(width));
        let bundle_bottom = bundle.y.min(bundle.y.saturating_add(height));

        // let clamped_width = bundle_right - bundle.x;

        // Process each visible scanline
        for y in bundle.y..bundle_bottom {
            let local_y = y - bundle.y;
            for x in bundle.x ..bundle_right {
                let local_x = x - bundle.x;
                let cluster_x = x as usize / 8 as usize;
                let tile_index = (local_y as usize * bundle.tile.w as usize) + local_x as usize;
                let tile_pixel = tile_clusters[tile_index];
                // let global_pixel =
                // self.lines[y as usize][cluster_x] =
            }
        }
    }

    pub fn clear(&mut self) {
        // Reset all scanlines
        for line in &mut self.lines {
            *line = from_fn(|_| PixelCluster::default());
        }
    }
}
