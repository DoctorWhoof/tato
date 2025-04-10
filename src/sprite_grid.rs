use crate::*;

#[derive(Debug, Clone)]
pub(crate) struct PixelQuery {
    pub pixel: SubPixel,
    pub flags: TileFlags,
}

/// A convenient packet of data used to draw a tile as a sprite.
#[derive(Debug, Clone, Copy)]
pub struct DrawBundle {
    pub x: i16,
    pub y: i16,
    pub id: TileID,
    pub flags: TileFlags,
}

/// A convenient packet of data to be passed in some functions.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct TileBundle {
    pub x: i16, // Original x position of the sprite
    pub y: i16, // Original y position of the sprite
    pub tile: TileEntry,
    pub flags: TileFlags,
}

/// Const LINE_COUNT is the total number of scanlines.
#[derive(Debug, Clone)]
pub(crate) struct SpriteGrid<const LINE_COUNT: usize> {
    // Each scanline can hold up to 8 sprite segments
    // LINE_COUNT is the maximum number of lines, height is the actual visible line count
    pub lines: [Scanline; LINE_COUNT],
    width: u16,
    height: u16,
}

impl<const LINE_COUNT: usize> SpriteGrid<LINE_COUNT> {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            lines: core::array::from_fn(|_| Scanline::default()),
            width,
            height,
        }
    }

    pub fn insert(&mut self, tile_pixels: &[PixelCluster], bundle: TileBundle) {
        const SEG_COUNT: usize = MAX_TILE_SIZE as usize / TILE_SIZE as usize;
        // Calculate effective sprite dimensions based on rotation
        let (width, height) = if bundle.flags.is_rotated() {
            (bundle.tile.h, bundle.tile.w) // Swap width and height for rotated sprites
        } else {
            (bundle.tile.w, bundle.tile.h)
        };

        // Calculate sprite boundaries in world coordinates
        let bundle_right = bundle.x + width as i16;
        let bundle_bottom = bundle.y + height as i16;

        // Skip if entirely off screen
        let left = bundle.x.max(0);
        let right = bundle_right.min(self.width as i16);
        if right <= left {
            return;
        }

        let top = bundle.y.max(0);
        let bottom = bundle_bottom.min(self.height as i16);
        if bottom <= top {
            return;
        }

        let clamped_width = right - left;

        // Process each visible scanline
        for y_pos in top as usize..bottom as usize {
            if y_pos >= LINE_COUNT {
                continue;
            }

            // Prepare the transformed scanline data
            let mut line_pixels = [PixelCluster::default(); SEG_COUNT];

            if bundle.flags.is_rotated() {
                // For 90° clockwise rotation:
                // Calculate local y position in the rotated sprite
                let local_y = y_pos as i16 - bundle.y;

                // Copy pixels for this scanline
                for seg_x in 0..clamped_width.min((SEG_COUNT * PIXELS_PER_CLUSTER) as i16) {
                    // Calculate local x position in the rotated sprite
                    let local_x = seg_x + (left - bundle.x);

                    // Then apply flipping if needed
                    let orig_x = if bundle.flags.is_flipped_y() {
                        (bundle.tile.h as i16 - 1) - local_y
                    } else {
                        local_y
                    };

                    let orig_y = if bundle.flags.is_flipped_x() {
                        local_x
                    } else {
                        (bundle.tile.w as i16 - 1) - local_x
                    };

                    // Calculate source index in the original tile data
                    let tile_width = bundle.tile.w as usize;
                    let clusters_per_row =
                        (tile_width + PIXELS_PER_CLUSTER - 1) / PIXELS_PER_CLUSTER;

                    let src_cluster_x = orig_x as usize / PIXELS_PER_CLUSTER;
                    let src_subpixel = orig_x as usize % PIXELS_PER_CLUSTER;
                    let source_idx = src_cluster_x + ((orig_y as usize) * clusters_per_row);

                    // Calculate destination position in the scanline
                    let dst_cluster = seg_x as usize / PIXELS_PER_CLUSTER;
                    let dst_subpixel = seg_x as usize % PIXELS_PER_CLUSTER;

                    // Copy the pixel if source is within bounds
                    if dst_cluster < SEG_COUNT && source_idx < tile_pixels.len() {
                        let pixel_value = tile_pixels[source_idx].get_subpixel(src_subpixel);
                        line_pixels[dst_cluster].set_subpixel(pixel_value, dst_subpixel);
                    }
                }
            } else {
                // For non-rotated sprites: normal row access with possible Y-flip
                let local_y = y_pos as i16 - bundle.y;
                let src_y = if bundle.flags.is_flipped_y() {
                    (bundle.tile.h as i16 - 1) - local_y
                } else {
                    local_y
                };

                // Copy pixels for this scanline with normal transformation
                for seg_x in 0..clamped_width.min((SEG_COUNT * PIXELS_PER_CLUSTER) as i16) {
                    let local_x = seg_x + (left - bundle.x);

                    // Apply horizontal flipping if needed
                    let src_x = if bundle.flags.is_flipped_x() {
                        (bundle.tile.w as i16 - 1) - local_x
                    } else {
                        local_x
                    };

                    // Calculate source index in the original tile data
                    let tile_width = bundle.tile.w as usize;
                    let clusters_per_row =
                        (tile_width + PIXELS_PER_CLUSTER - 1) / PIXELS_PER_CLUSTER;
                    let src_cluster_x = src_x as usize / PIXELS_PER_CLUSTER;
                    let src_subpixel = src_x as usize % PIXELS_PER_CLUSTER;
                    let source_idx = src_cluster_x + ((src_y as usize) * clusters_per_row);

                    // Calculate destination position in the scanline
                    let dst_cluster = seg_x as usize / PIXELS_PER_CLUSTER;
                    let dst_subpixel = seg_x as usize % PIXELS_PER_CLUSTER;

                    // Copy the pixel if source is within bounds
                    if dst_cluster < SEG_COUNT && source_idx < tile_pixels.len() {
                        let pixel_value = tile_pixels[source_idx].get_subpixel(src_subpixel);
                        line_pixels[dst_cluster].set_subpixel(pixel_value, dst_subpixel);
                    }
                }
            }

            // Now insert the prepared scanline
            let scanline = &mut self.lines[y_pos];
            let mini_rect = MiniRect {
                x: left,
                w: clamped_width,
            };
            scanline.insert(line_pixels, left, bundle.flags, mini_rect);
        }
    }

    pub fn clear(&mut self) {
        // Reset all scanlines
        for line in &mut self.lines {
            line.clear()
        }
    }
}
