use crate::*;

/// Limit of sprites per scanline. Once reached, no more sprites can be drawn in this line.
pub const MAX_SPRITES_PER_LINE: usize = 8;

/// How many pixels long is a scanline slot
pub(crate) const SLOT_WIDTH: u16 = FG_WIDTH / 32;

/// How many bytes a segment takes, accounting for Pixel cluster size
/// A pixel cluster contains TILE_SIZE pixels.
const SEG_COUNT: usize = MAX_TILE_SIZE as usize / TILE_SIZE as usize; // in bytes

#[derive(Debug, Clone, Copy, Default)]
struct Segment {
    source_x: i16, // Original x position of the sprite
    flags: TileFlags,
    pixels: [PixelCluster; SEG_COUNT],
}

#[derive(Debug, Clone)]
pub(crate) struct Scanline {
    pub(crate) bit_slots: u32, // Bit field for 32 horizontal sections
    segments: [Segment; MAX_SPRITES_PER_LINE], // Array of sprite segments
    count: u8,                 // Count of active segments
}

impl Default for Scanline {
    fn default() -> Self {
        Scanline {
            count: 0,
            bit_slots: 0,
            segments: [Segment::default(); MAX_SPRITES_PER_LINE],
        }
    }
}

pub(crate) struct MiniRect {
    pub x: i16,
    pub w: i16,
}

impl Scanline {
    #[inline]
    pub fn clear(&mut self) {
        self.count = 0;
        self.bit_slots = 0;
        // No need to reset the segments themselves
    }

    #[inline]
    pub fn insert(
        &mut self,
        line_pixels: [PixelCluster; SEG_COUNT], // Pre-transformed scanline data
        source_x: i16,                          // X position on screen
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
    pub fn get(&self, screen_x: u16) -> Option<PixelQuery> {
        // Determine which bit in bit_slots corresponds to this x position
        let section = screen_x / SLOT_WIDTH;

        // If this section has no sprites (bit is 0), return early
        if ((self.bit_slots >> section) & 1) == 0 {
            return None;
        }

        // If we have sprites in this section, check each one
        for i in 0..self.count as usize {
            let segment = &self.segments[i];

            // Calculate local coordinates within the sprite
            let local_x = (screen_x as i16 - segment.source_x) as usize;

            // Get the pixel using the correct cluster size
            let cluster_idx = local_x / PIXELS_PER_CLUSTER;
            if cluster_idx >= SEG_COUNT {
                continue;
            }
            let subpixel_idx = local_x % PIXELS_PER_CLUSTER;
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
