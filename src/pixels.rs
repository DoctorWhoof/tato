use crate::*;

// Constants
pub const BITS_PER_PIXEL: u8 = 2;
pub const PIXELS_PER_CLUSTER: usize = PixelCluster::<u16>::pixel_count(); // For u16 with 2 bits per pixel

// A distinct subpixel type
#[derive(Debug, Clone, Copy, Default)]
pub struct SubPixel(pub u8);

/// Generic PixelCluster that can use different integer types.
/// With u16 and 2 bits per pixel each cluster holds 8 pixels.
#[derive(Debug, Clone, Copy, Default)]
pub struct PixelCluster<T: IntegerOps = u16> {
    data: T,
}

impl<T> PixelCluster<T>
where
    T: IntegerOps,
{
    // Calculate max pixels this type can store
    #[inline(always)]
    pub const fn pixel_count() -> usize {
        T::BITS / BITS_PER_PIXEL as usize
    }

    #[inline(always)]
    pub fn get_subpixel(&self, sub_index: usize) -> SubPixel {
        #[cfg(debug_assertions)]
        {
            assert!(sub_index < Self::pixel_count(), "Pixel index out of bounds");
        }

        // Calculate bit position
        let bit_start = sub_index * BITS_PER_PIXEL as usize;
        let shift = T::BITS - bit_start - BITS_PER_PIXEL as usize;

        // Create mask
        let mask = T::from_u8((1 << BITS_PER_PIXEL) - 1);

        // Extract value
        let value = (self.data >> shift) & mask;
        SubPixel(value.to_u8())
    }

    #[inline(always)]
    pub fn set_subpixel(&mut self, value: SubPixel, sub_index: usize) {
        #[cfg(debug_assertions)]
        {
            assert!(sub_index < Self::pixel_count(), "Pixel index out of bounds");
        }

        // Calculate bit position
        let bit_start = sub_index * BITS_PER_PIXEL as usize;
        let shift = T::BITS - bit_start - BITS_PER_PIXEL as usize;

        // Create mask
        let mask = T::from_u8((1 << BITS_PER_PIXEL) - 1);

        // Clear bits
        self.data = self.data & !(mask << shift);

        // Set bits
        let val = T::from_u8(value.0 & ((1 << BITS_PER_PIXEL) - 1));
        self.data = self.data | (val << shift);
    }
}

// #[inline(always)]
// pub const fn get_subpixel(cluster: u8, subpixel: usize) -> u8 {
//     let bit_position = 6 - ((subpixel % 4) * 2);
//     (cluster >> bit_position) & 0x03
// }

// #[inline(always)]
// pub const fn set_subpixel(cluster: &mut u8, value: u8, subpixel: usize) {
//     let bit_position = 6 - ((subpixel % 4) * 2);
//     let value = value & 0x03;
//     // Clear the bits
//     *cluster &= !(0x03 << bit_position);
//     // Set the bits
//     *cluster |= value << bit_position;
// }

// Versions that accept different bit counts per pixel!

// use error::err;
// #[inline]
// pub const fn get_subpixel(pixel_cluster: u8, bits_per_pixel: u8, pixel_index: usize) -> u8 {
//     match bits_per_pixel {
//         1 => {
//             let bit_position = 7 - (pixel_index % 8);
//             (pixel_cluster >> bit_position) & 0x01
//         }
//         2 => {
//             let bit_position = 6 - ((pixel_index % 4) * 2);
//             (pixel_cluster >> bit_position) & 0x03
//         }
//         4 => {
//             let nibble_position = (pixel_index % 2) * 4;
//             (pixel_cluster >> (4 - nibble_position)) & 0x0F
//         }
//         8 => pixel_cluster,
//         _ => panic!(err!("Unsupported bits per pixel value")),
//     }
// }

// #[inline]
// pub const fn set_subpixel(pixel_cluster: &mut u8, bits_per_pixel: u8, value: u8, pixel_index: usize) {
//     match bits_per_pixel {
//         1 => {
//             let bit_position = 7 - (pixel_index % 8);
//             let value = value & 0x01;
//             // Clear the bit
//             *pixel_cluster &= !(0x01 << bit_position);
//             // Set the bit
//             *pixel_cluster |= value << bit_position;
//         }
//         2 => {
//             let bit_position = 6 - ((pixel_index % 4) * 2);
//             let value = value & 0x03;
//             // Clear the bits
//             *pixel_cluster &= !(0x03 << bit_position);
//             // Set the bits
//             *pixel_cluster |= value << bit_position;
//         }
//         4 => {
//             let nibble_position = (pixel_index % 2) * 4;
//             let shift = 4 - nibble_position;
//             let value = value & 0x0F;
//             // Clear the nibble
//             *pixel_cluster &= !(0x0F << shift);
//             // Set the nibble
//             *pixel_cluster |= value << shift;
//         }
//         8 => {
//             *pixel_cluster = value;
//         }
//         _ => panic!(err!("Unsupported bits per pixel value")),
//     }
// }
