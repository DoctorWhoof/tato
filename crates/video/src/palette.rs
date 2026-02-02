#[derive(Clone, Default, Copy, PartialEq, Hash)]
pub struct Palette(pub u16);

impl Palette {
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        assert!(a < 16, "Palette: value 'a' exceed maximum of 15");
        assert!(b < 16, "Palette: value 'b' exceed maximum of 15");
        assert!(c < 16, "Palette: value 'c' exceed maximum of 15");
        assert!(d < 16, "Palette: value 'd' exceed maximum of 15");
        let data = (a as u16) << 12 | (b as u16) << 8 | (c as u16) << 4 | d as u16;
        Self(data)
    }

    pub const fn default() -> Self {
        Self::new(0, 1, 2, 3)
    }

    pub fn get(&self, slot: u8) -> u8 {
        match slot {
            0 => ((self.0 >> 12) & 15) as u8,
            1 => ((self.0 >> 8) & 15) as u8,
            2 => ((self.0 >> 4) & 15) as u8,
            3 => ((self.0) & 15) as u8,
            _ => panic!("Palette: Max index is 3, {} was provided", slot),
        }
    }

    pub fn set(&mut self, slot: u8, value: u8) {
        assert!(slot < 4, "Palette: Max index is 3, {} was provided", slot);
        assert!(value < 16, "Palette: Max slot value is 15, {} was provided", value);
        let shift = (3 - slot) as u16 * 4;
        let slot_mask = 15 << shift;
        let value_shifted = (value as u16) << shift;
        self.0 = (self.0 & !slot_mask) | value_shifted;
    }

    pub fn cycle(&mut self, start_index: u8, end_index: u8, delta: i8) {
        let arr: [u8; 4] = self.clone().into();
        let range_size = (end_index - start_index + 1) as isize;

        for i in start_index..=end_index {
            let current_pos = (i - start_index) as isize;
            let mut source_pos = (current_pos - delta as isize) % range_size;
            if source_pos < 0 {
                source_pos += range_size;
            }
            let source_index = start_index + source_pos as u8;
            self.set(i, arr[source_index as usize]);
        }
    }

}

impl From<[u8; 4]> for Palette {
    fn from(values: [u8; 4]) -> Self {
        Self::new(values[0], values[1], values[2], values[3])
    }
}

impl From<(u8, u8, u8, u8)> for Palette {
    fn from(values: (u8, u8, u8, u8)) -> Self {
        Self::new(values.0, values.1, values.2, values.3)
    }
}

impl Into<[u8; 4]> for Palette {
    fn into(self) -> [u8; 4] {
        [
            ((self.0 >> 12) & 15) as u8, //
            ((self.0 >> 8) & 15) as u8,
            ((self.0 >> 4) & 15) as u8,
            ((self.0) & 15) as u8,
        ]
    }
}

impl Into<(u8, u8, u8, u8)> for Palette {
    fn into(self) -> (u8, u8, u8, u8) {
        (
            ((self.0 >> 12) & 15) as u8, //
            ((self.0 >> 8) & 15) as u8,
            ((self.0 >> 4) & 15) as u8,
            ((self.0) & 15) as u8,
        )
    }
}

impl core::fmt::Debug for Palette {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let a = (self.0 >> 12) & 15;
        let b = (self.0 >> 8) & 15;
        let c = (self.0 >> 4) & 15;
        let d = self.0 & 15;
        write!(f, "Palette([{}, {}, {}, {}])", a, b, c, d)
    }
}
