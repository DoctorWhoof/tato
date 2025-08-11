use crate::*;

/// Every color in the main palettes (FG and BG palette) is stored as 3 bits-per-channel,
/// allowing a maximum of 512 possible colors packed into 12 bits (excluding alpha).
/// Includes a 4-bit z-buffer for rendering priority (0-15, higher values render in front).
/// Can be converted to RGBA32 (8 bits per channel) for easy interop with graphics back-ends.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct RGBA12 {
    pub data: u16,
}

impl RGBA12 {
    pub const TRANSPARENT: RGBA12 = RGBA12::with_transparency(0, 0, 0, 0);
    pub const BLACK: RGBA12 = RGBA12::new(0, 0, 0);
    pub const GRAY: RGBA12 = RGBA12::new(4, 4, 4);
    pub const WHITE: RGBA12 = RGBA12::new(7, 7, 7);
    pub const DARK_RED: RGBA12 = RGBA12::new(3, 0, 0);
    pub const RED: RGBA12 = RGBA12::new(5, 2, 2);
    pub const LIGHT_RED: RGBA12 = RGBA12::new(7, 5, 5);
    pub const ORANGE: RGBA12 = RGBA12::new(6, 4, 1);
    pub const YELLOW: RGBA12 = RGBA12::new(7, 6, 3);
    pub const DARK_GREEN: RGBA12 = RGBA12::new(0, 2, 1);
    pub const GREEN: RGBA12 = RGBA12::new(2, 4, 2);
    pub const LIGHT_GREEN: RGBA12 = RGBA12::new(4, 6, 3);
    pub const DARK_BLUE: RGBA12 = RGBA12::new(0, 1, 3);
    pub const BLUE: RGBA12 = RGBA12::new(1, 2, 6);
    pub const LIGHT_BLUE: RGBA12 = RGBA12::new(4, 6, 7);
    pub const PINK: RGBA12 = RGBA12::new(6, 3, 6);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        assert!(r < 8, err!("Exceeded maximum value for Red channel"));
        assert!(g < 8, err!("Exceeded maximum value for Green channel"));
        assert!(b < 8, err!("Exceeded maximum value for Blue channel"));
        // Pack the 3-bit color values into the data field (z-buffer defaults to 0)
        // Z in bits 12-15, Alpha in bits 9-11, Red in bits 6-8, Green in bits 3-5, Blue in bits 0-2
        let packed_data = ((7 as u16) << 9) | ((r as u16) << 6) | ((g as u16) << 3) | (b as u16);
        Self { data: packed_data }
    }

    pub const fn with_transparency(r: u8, g: u8, b: u8, a:u8) -> Self {
        assert!(r < 8, err!("Exceeded maximum value for Red channel"));
        assert!(g < 8, err!("Exceeded maximum value for Green channel"));
        assert!(b < 8, err!("Exceeded maximum value for Blue channel"));
        assert!(a < 8, err!("Exceeded maximum value for Alpha channel"));
        // Pack the 3-bit color values into the data field (z-buffer defaults to 0)
        // Z in bits 12-15, Alpha in bits 9-11, Red in bits 6-8, Green in bits 3-5, Blue in bits 0-2
        let packed_data = ((a as u16) << 9) | ((r as u16) << 6) | ((g as u16) << 3) | (b as u16);
        Self { data: packed_data }
    }

    pub fn r(&self) -> u8 {
        (self.data >> 6 & 0b_0111) as u8
    }

    pub fn g(&self) -> u8 {
        (self.data >> 3 & 0b_0111) as u8
    }

    pub fn b(&self) -> u8 {
        (self.data & 0b_0111) as u8
    }

    pub fn a(&self) -> u8 {
        (self.data >> 9 & 0b_0111) as u8
    }

    pub fn z(&self) -> u8 {
        (self.data >> 12 & 0b_1111) as u8
    }

    pub fn set_r(&mut self, r: u8) {
        assert!(r < 8, err!("Exceeded maximum value for Red channel"));
        self.data = (self.data & !(0b_0111 << 6)) | ((r as u16) << 6);
    }

    pub fn set_g(&mut self, g: u8) {
        assert!(g < 8, err!("Exceeded maximum value for Green channel"));
        self.data = (self.data & !(0b_0111 << 3)) | ((g as u16) << 3);
    }

    pub fn set_b(&mut self, b: u8) {
        assert!(b < 8, err!("Exceeded maximum value for Blue channel"));
        self.data = (self.data & !(0b_0111)) | (b as u16);
    }

    pub fn set_a(&mut self, a: u8) {
        assert!(a < 8, err!("Exceeded maximum value for Alpha channel"));
        self.data = (self.data & !(0b_0111 << 9)) | ((a as u16) << 9);
    }

    pub fn set_z(&mut self, z: u8) {
        assert!(z < 16, err!("Z value must be less than 16 (4 bits)"));
        self.data = (self.data & !(0b_1111 << 12)) | ((z as u16) << 12);
    }

    pub fn with_z(self, z: u8) -> Self {
        let mut result = self;
        result.set_z(z);
        result
    }
}

impl From<RGBA12> for RGBA32 {
    fn from(color: RGBA12) -> Self {
        // Extract the 3-bit color components
        let r = ((color.data >> 6) & 0x7) as u8;
        let g = ((color.data >> 3) & 0x7) as u8;
        let b = (color.data & 0x7) as u8;
        let a = ((color.data >> 9) & 0x7) as u8;
        // Scale the 3-bit values (0-7) to 8-bit range (0-255)
        Self {
            // Approximate v * 36.4 without overflow
            r: (r * 36) + (r / 2),
            g: (g * 36) + (g / 2),
            b: (b * 36) + (b / 2),
            a: (a * 36) + (a / 2),
        }
    }
}

impl From<RGBA32> for RGBA12 {
    fn from(color: RGBA32) -> Self {
        // Scale down 8-bit values (0-255) to 3-bit values (0-7)
        let r = (color.r / 36).min(7) as u16;
        let g = (color.g / 36).min(7) as u16;
        let b = (color.b / 36).min(7) as u16;
        let a = (color.a / 36).min(7) as u16;
        // Combine the 3-bit components into a 12-bit value
        Self {
            data: (a << 9) | (r << 6) | (g << 3) | b,
        }
    }
}

impl core::fmt::Display for RGBA12 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "RGBA12(r: {}, g: {}, b: {}, a: {}, z: {})",
            self.r(),
            self.g(),
            self.b(),
            self.a(),
            self.z()
        )
    }
}
