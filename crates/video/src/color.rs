mod rgba12;
pub use rgba12::*;

mod rgba32;
pub use rgba32::*;

/// Unique identifier for a color in the Main Palettes.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct ColorID(pub u8);

impl ColorID {
    pub fn id(self) -> usize {
        self.0 as usize
    }
}

impl From<ColorID> for u8 {
    fn from(value: ColorID) -> Self {
        value.0
    }
}
