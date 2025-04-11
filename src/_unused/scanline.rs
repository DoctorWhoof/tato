use crate::*;

const CLUSTERS_PER_LINE: usize = 256 / PIXELS_PER_CLUSTER as usize;

#[derive(Debug, Clone)]
pub(crate) struct Scanline {
    data: [PixelCluster; CLUSTERS_PER_LINE],
}

impl Default for Scanline {
    fn default() -> Self {
        Self {
            data: core::array::from_fn(|_| PixelCluster::default()),
        }
    }
}

impl Scanline {
    #[inline]
    pub fn clear(&mut self) {
        self.data = core::array::from_fn(|_| PixelCluster::default());
    }

    #[inline]
    pub fn insert(
        &mut self,
        line_pixels: [PixelCluster; SEG_COUNT], // Pre-transformed scanline data
        source_x: u8,                          // X position on screen
        flags: TileFlags,                       // Keep flags for palette info
        clamped_rect: MiniRect,                 // For bounds info
    ) {
        // Check if we have space for another segment
        if self.count as usize >= MAX_SPRITES_PER_LINE {
            return;
        }

        // Add segment
        let i = self.count as usize;

        self.segments[i] = Segment {
            pixels: line_pixels,
            flags,
            source_x,
        };
        self.count += 1;

        // Set the section bits (dividing screen into 32 sections)
        let start_section = clamped_rect.x as usize / 8; // 256/32 = 8 pixels per section
        let end_section = ((clamped_rect.x + clamped_rect.w - 1) as usize) / 8;

        for section in start_section..=end_section.min(31) {
            // Each bit set to 1 indicates "there's at least one sprite in this segment"
            self.bit_slots |= 1 << section;
        }
    }

    #[inline]
    pub fn get(&self, screen_x: u8) -> Option<PixelQuery> {
        // Determine which bit in bit_slots corresponds to this x position
        let section = screen_x as u16 / SLOT_WIDTH;

        // If this section has no sprites (bit is 0), return early
        if ((self.bit_slots >> section) & 1) == 0 {
            return None;
        }

        // If we have sprites in this section, check each one
        for i in 0..self.count as usize {
            let segment = &self.segments[i];

            // Calculate local coordinates within the sprite
            if screen_x < segment.source_x { continue; }
            let local_x = (screen_x - segment.source_x) as usize;

            // Get the pixel using the correct cluster size
            let cluster_idx = local_x / PIXELS_PER_CLUSTER as usize;
            if cluster_idx >= SEG_COUNT {
                continue;
            }
            let subpixel_idx = local_x % PIXELS_PER_CLUSTER as usize;
            let pixel = segment.pixels[cluster_idx].get_subpixel(subpixel_idx);

            // If the pixel is non-transparent, return it
            if pixel.0 != 0 {
                return Some(PixelQuery {
                    pixel,
                    flags: segment.flags,
                });
            }
        }

        // No non-transparent sprite found at this position
        None
    }
}
