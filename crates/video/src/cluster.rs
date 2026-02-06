use crate::{TileFlags, err};

/// A cluster always contains 8 pixels, regardless of color depth.
pub const PIXELS_PER_CLUSTER: u8 = 8;

/// A Cluster always stores 8 pixels, and simply gets larger the more colors you store in it.
/// At 2 bits per pixel (4 colors) it is 2 bytes.
/// Since we always have 8 bits per pixel, BITS_PER_PIXEL is also the number of bytes!
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct Cluster<const BITS_PER_PIXEL: usize> {
    pub data: [u8; BITS_PER_PIXEL],
}

impl<const BITS_PER_PIXEL: usize> Default for Cluster<BITS_PER_PIXEL> {
    fn default() -> Self {
        assert!(BITS_PER_PIXEL > 0 && BITS_PER_PIXEL < 9, err!("Invalid BITS_PER_PIXEL"));
        Self { data: [0; BITS_PER_PIXEL] }
    }
}

impl<const BITS_PER_PIXEL: usize> Cluster<BITS_PER_PIXEL> {
    // How many pixels fit in each byte
    pub const PIXELS_PER_BYTE: usize = PIXELS_PER_CLUSTER as usize / BITS_PER_PIXEL;
    pub const BYTES_PER_CLUSTER: usize = BITS_PER_PIXEL; // works out this way for 8 pixel clusters
    // Mask for extracting a single pixel value
    pub const MASK: u8 = (1 << BITS_PER_PIXEL) - 1;

    #[inline(always)]
    pub fn get_subpixel(&self, index: u8) -> u8 {
        debug_assert!(index < PIXELS_PER_CLUSTER, err!("Pixel index out of bounds"));

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
        debug_assert!(index < PIXELS_PER_CLUSTER, err!("Pixel index out of bounds"));

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
        let mut flipped = Self { data: [0; BITS_PER_PIXEL] };

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
    pub fn from_tile(
        tile_pixels: &[Cluster<BITS_PER_PIXEL>],
        flags: TileFlags,
        row: u8,
        tile_size: u8,
    ) -> Self {
        debug_assert!(row < tile_size, "Row index out of bounds");

        let mut result = Self::default();

        // For each pixel position in the output row
        for col in 0..tile_size {
            // Use transform_coords to find which source pixel to read
            let (src_col, src_row) = flags.transform_coords(col, row, tile_size);

            // Get the pixel from the source tile
            let src_cluster = &tile_pixels[src_row as usize];
            let pixel_value = src_cluster.get_subpixel(src_col);

            // Set in output cluster
            result.set_subpixel(pixel_value, col);
        }

        result
    }

    pub fn from_pixels(pixels: &[u8]) -> Self {
        assert!(pixels.len() == 8, err!("Length of pixels array must be 8 to convert to Cluster"));
        let mut cluster = Self::default();
        for (i, &pixel) in pixels.iter().enumerate() {
            cluster.set_subpixel(pixel, i as u8);
        }
        cluster
    }
}

impl<const BITS_PER_PIXEL: usize> From<&[u8]> for Cluster<BITS_PER_PIXEL> {
    fn from(pixels: &[u8]) -> Self {
        Self::from_pixels(pixels)
    }
}

// impl<const BITS_PER_PIXEL: usize> TryFrom<&[u8]> for Cluster<BITS_PER_PIXEL> {
//     type Error = &'static str;

//     fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
//         if slice.len() != 8 {
//             return Err("Input slice must be exactly 8 pixels long");
//         }

//         let pixels: [u8; 8] = slice.try_into().unwrap();
//         Ok(Self::from_pixels(pixels))
//     }
// }
