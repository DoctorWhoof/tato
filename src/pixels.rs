pub const PIXELS_PER_CLUSTER:u8 = 8;

/// A PixelCluster always stores 8 pixels
#[derive(Clone, Copy)]
pub struct PixelCluster<const BYTES: usize> {
    // Raw storage - always 8 pixels
    data: [u8; BYTES],
}

impl<const BYTES: usize> Default for PixelCluster<BYTES> {
    fn default() -> Self {
        Self {
            data: [0; BYTES],
        }
    }
}

impl<const BYTES: usize> PixelCluster<BYTES> {
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
}

impl<const BYTES: usize> core::fmt::Debug for PixelCluster<BYTES> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

impl<const BYTES: usize> core::fmt::Display for PixelCluster<BYTES> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}
