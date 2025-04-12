pub const PIXELS_PER_CLUSTER: u8 = 8;

/// A Cluster always stores 8 pixels
#[derive(Clone, Copy)]
pub struct Cluster<const BYTES: usize> {
    // Raw storage - always 8 pixels
    data: [u8; BYTES],
}

impl<const BYTES: usize> Default for Cluster<BYTES> {
    fn default() -> Self {
        Self { data: [0; BYTES] }
    }
}

impl<const BYTES: usize> Cluster<BYTES> {
    // How many pixels fit in each byte
    pub const PIXELS_PER_BYTE: usize = PIXELS_PER_CLUSTER as usize / BYTES;
    // How many bits each pixel uses
    pub const BITS_PER_PIXEL: usize = PIXELS_PER_CLUSTER as usize / Self::PIXELS_PER_BYTE;
    // Mask for extracting a single pixel value
    pub const MASK: u8 = (1 << Self::BITS_PER_PIXEL) - 1;

    #[inline(always)]
    pub fn get_subpixel(&self, index: u8) -> u8 {
        debug_assert!(index < PIXELS_PER_CLUSTER, "Pixel index out of bounds");

        // Calculate which byte contains this pixel
        let byte_idx = index as usize / Self::PIXELS_PER_BYTE;

        // Calculate position within the byte
        let pos_in_byte = index as usize % Self::PIXELS_PER_BYTE;
        let shift = (Self::PIXELS_PER_BYTE - 1 - pos_in_byte) * Self::BITS_PER_PIXEL;

        // Extract the pixel value
        (self.data[byte_idx] >> shift) & Self::MASK
    }

    #[inline(always)]
    pub fn set_subpixel(&mut self, value: u8, index: u8) {
        debug_assert!(index < PIXELS_PER_CLUSTER, "Pixel index out of bounds");

        // Limit value to valid range
        let value = value & Self::MASK;

        // Calculate which byte contains this pixel
        let byte_idx = index as usize / Self::PIXELS_PER_BYTE;

        // Calculate position within the byte
        let pos_in_byte = index as usize % Self::PIXELS_PER_BYTE;
        let shift = (Self::PIXELS_PER_BYTE - 1 - pos_in_byte) * Self::BITS_PER_PIXEL;

        // Clear the bits for this pixel
        let mask = Self::MASK << shift;
        self.data[byte_idx] &= !mask;

        // Set the new value
        self.data[byte_idx] |= value << shift;
    }

    pub fn flip(&self) -> Self {
        let mut flipped = Self { data: [0; BYTES] };

        let mut left_pixel = 0;
        let mut right_pixel = PIXELS_PER_CLUSTER - 1;

        while left_pixel < right_pixel {
            // Calculate positions for left pixel
            let left_byte = left_pixel as usize / Self::PIXELS_PER_BYTE;
            let left_pos = left_pixel as usize % Self::PIXELS_PER_BYTE;
            let left_shift = (Self::PIXELS_PER_BYTE - 1 - left_pos) * Self::BITS_PER_PIXEL;

            // Calculate positions for right pixel
            let right_byte = right_pixel as usize / Self::PIXELS_PER_BYTE;
            let right_pos = right_pixel as usize % Self::PIXELS_PER_BYTE;
            let right_shift = (Self::PIXELS_PER_BYTE - 1 - right_pos) * Self::BITS_PER_PIXEL;

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
