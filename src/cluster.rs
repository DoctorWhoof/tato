use crate::{TileFlags, err};

/// A cluster always contains 8 pixels, regardless of color depth.
pub const PIXELS_PER_CLUSTER: u8 = 8;

/// A Cluster always stores 8 pixels, and simply gets larger the more colors you store in it.
/// At 2 bits per pixel (4 colors) it is 2 bytes.
/// Since we always have 8 bits per pixel, BITS_PER_PIXEL is also the number of bytes!
#[derive(Clone, Copy)]
pub struct Cluster<const BITS_PER_PIXEL: usize> {
    data: [u8; BITS_PER_PIXEL],
}

impl<const BITS_PER_PIXEL: usize> Default for Cluster<BITS_PER_PIXEL> {
    fn default() -> Self {
        assert!(
            BITS_PER_PIXEL > 0 && BITS_PER_PIXEL < 9,
            err!("Invalid BITS_PER_PIXEL")
        );
        Self {
            data: [0; BITS_PER_PIXEL],
        }
    }
}

impl<const BITS_PER_PIXEL: usize> Cluster<BITS_PER_PIXEL> {
    // How many pixels fit in each byte
    pub const PIXELS_PER_BYTE: usize = PIXELS_PER_CLUSTER as usize / BITS_PER_PIXEL;
    // Mask for extracting a single pixel value
    pub const MASK: u8 = (1 << BITS_PER_PIXEL) - 1;

    #[inline(always)]
    pub fn get_subpixel(&self, index: u8) -> u8 {
        debug_assert!(
            index < PIXELS_PER_CLUSTER,
            err!("Pixel index out of bounds")
        );

        // Calculate which byte contains this pixel
        let byte_idx = index as usize / Self::PIXELS_PER_BYTE;

        // Calculate position within the byte
        let pos_in_byte = index as usize % Self::PIXELS_PER_BYTE;
        let shift = (Self::PIXELS_PER_BYTE - 1 - pos_in_byte) * BITS_PER_PIXEL;

        // Extract the pixel value
        (self.data[byte_idx] >> shift) & Self::MASK
    }

    #[inline(always)]
    pub fn set_subpixel(&mut self, value: u8, index: u8) {
        debug_assert!(
            index < PIXELS_PER_CLUSTER,
            err!("Pixel index out of bounds")
        );

        // Limit value to valid range
        let value = value & Self::MASK;

        // Calculate which byte contains this pixel
        let byte_idx = index as usize / Self::PIXELS_PER_BYTE;

        // Calculate position within the byte
        let pos_in_byte = index as usize % Self::PIXELS_PER_BYTE;
        let shift = (Self::PIXELS_PER_BYTE - 1 - pos_in_byte) * BITS_PER_PIXEL;

        // Clear the bits for this pixel
        let mask = Self::MASK << shift;
        self.data[byte_idx] &= !mask;

        // Set the new value
        self.data[byte_idx] |= value << shift;
    }

    #[inline]
    pub fn flip(&self) -> Self {
        let mut flipped = Self {
            data: [0; BITS_PER_PIXEL],
        };

        let mut left_pixel = 0;
        let mut right_pixel = PIXELS_PER_CLUSTER - 1;

        while left_pixel < right_pixel {
            // Calculate positions for left pixel
            let left_byte = left_pixel as usize / Self::PIXELS_PER_BYTE;
            let left_pos = left_pixel as usize % Self::PIXELS_PER_BYTE;
            let left_shift = (Self::PIXELS_PER_BYTE - 1 - left_pos) * BITS_PER_PIXEL;

            // Calculate positions for right pixel
            let right_byte = right_pixel as usize / Self::PIXELS_PER_BYTE;
            let right_pos = right_pixel as usize % Self::PIXELS_PER_BYTE;
            let right_shift = (Self::PIXELS_PER_BYTE - 1 - right_pos) * BITS_PER_PIXEL;

            // Extract pixel values
            let left_val = (self.data[left_byte] >> left_shift) & Self::MASK;
            let right_val = (self.data[right_byte] >> right_shift) & Self::MASK;

            // Set pixels in the flipped version (swapped)
            let left_mask = Self::MASK << left_shift;
            flipped.data[left_byte] &= !left_mask;
            flipped.data[left_byte] |= right_val << left_shift;

            let right_mask = Self::MASK << right_shift;
            flipped.data[right_byte] &= !right_mask;
            flipped.data[right_byte] |= left_val << right_shift;

            // Move indices toward center
            left_pixel += 1;
            right_pixel -= 1;
        }

        flipped
    }

    #[inline]
    pub fn from_tile(tile_pixels: &[Cluster<BITS_PER_PIXEL>], flags: TileFlags, row: u8, tile_size:u8) -> Self {
        debug_assert!(row < 8, "Row index out of bounds");

        if flags.is_rotated() {
            // For 90° clockwise rotation
            let mut rotated = Self::default();
            let high = tile_size - 1;

            // First, determine which column we need
            let col = if flags.is_flipped_y() {
                // If flipped_y is set after rotation, it flips the columns
                row
            } else {
                // Normal rotation makes column = 7 - row
                high - row
            };

            // Now, determine the reading direction for the column
            let start_row = if flags.is_flipped_x() {
                // If flipped_x is set after rotation, read column from bottom to top
                high
            } else {
                // Otherwise, read column from top to bottom
                0
            };

            let row_step = if flags.is_flipped_x() { -1_i8 } else { 1_i8 };

            // Fill the destination cluster with pixels from the column
            for i in 0..tile_size {
                // Calculate the source row
                let src_row = (start_row as i8 + (i as i8 * row_step)) as u8 & 7;

                // Get the pixel from the column in the source tile
                let src_cluster = tile_pixels[src_row as usize];
                let pixel_value = src_cluster.get_subpixel(col);

                // Set in our output cluster
                rotated.set_subpixel(pixel_value, i);
            }

            rotated
        } else {
            // No transformations - return the source cluster directly
            if !flags.is_flipped_x() && !flags.is_flipped_y() {
                return tile_pixels[row as usize];
            }
            // No rotation, just handle flipping
            let source_row = if flags.is_flipped_y() { 7 - row } else { row };
            let source_cluster = tile_pixels[source_row as usize];
            if flags.is_flipped_x() {
                source_cluster.flip()
            } else {
                source_cluster
            }
        }
    }
}

impl<const BYTES: usize> core::fmt::Debug for Cluster<BYTES> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

impl<const BYTES: usize> core::fmt::Display for Cluster<BYTES> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}
