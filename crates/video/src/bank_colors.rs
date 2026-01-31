use crate::*;
use core::array::from_fn;

pub type PaletteRemap = [u8; COLORS_PER_PALETTE as usize];
pub const DEFAULT_PALETTE: [RGBA12; COLORS_PER_PALETTE as usize] = [
    RGBA12::TRANSPARENT, // 0
    RGBA12::BLACK,       // 1
    RGBA12::GRAY,        // 2
    RGBA12::WHITE,       // 3
    RGBA12::DARK_RED,    // 4
    RGBA12::RED,         // 5
    RGBA12::LIGHT_RED,   // 6
    RGBA12::ORANGE,      // 7
    RGBA12::YELLOW,      // 8
    RGBA12::DARK_GREEN,  // 9
    RGBA12::GREEN,       // 10
    RGBA12::LIGHT_GREEN, // 11
    RGBA12::DARK_BLUE,   // 12
    RGBA12::BLUE,        // 13
    RGBA12::LIGHT_BLUE,  // 14
    RGBA12::PINK,        // 15
];

pub const DEFAULT_MAPPING: [u8; COLORS_PER_PALETTE as usize] =
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

#[derive(Debug, Clone)]
pub struct ColorBank {
    pub palette: [RGBA12; COLORS_PER_PALETTE as usize],
    pub(crate) palette_head: u8,
}

impl ColorBank {
    pub const fn new() -> Self {
        Self {
            palette: DEFAULT_PALETTE,
            palette_head: 0,
        }
    }

    pub const fn new_from(
        palette: &[RGBA12],
    ) -> Self {
        // Create palette array
        let mut palette_array = [RGBA12::new(0, 0, 0); COLORS_PER_PALETTE as usize];
        let mut i = 0;
        while i < COLORS_PER_PALETTE as usize {
            if i < palette.len() {
                palette_array[i] = palette[i];
            }
            i += 1;
        }

        Self {
            palette: palette_array,
            palette_head: palette.len() as u8,
        }
    }

    pub fn color_count(&self) -> u8 {
        self.palette_head
    }

    pub fn reset_palettes(&mut self) {
        self.palette = PALETTE_DEFAULT;
        self.palette_head = 0;
    }

    /// Restore palette counters to previous state (for checkpoint/restore)
    /// Warning: Caller must ensure these are valid previous states!
    pub fn restore_state(&mut self, color_count: u8, color_mapping_count: u8) {
        assert!(color_count <= COLORS_PER_PALETTE as u8, "Invalid color count");
        assert!(color_mapping_count <= 16, "Invalid color mapping count");
        self.palette_head = color_count;
    }

    pub fn push_color(&mut self, color: RGBA12) -> ColorID {
        assert!(self.palette_head < COLORS_PER_PALETTE as u8, "Palette capacity reached");
        let id = ColorID(self.palette_head);
        self.palette[self.palette_head as usize] = color;
        self.palette_head += 1;
        id
    }

    pub fn load_default(&mut self) {
        self.palette = [
            RGBA12::TRANSPARENT, // 0
            RGBA12::BLACK,       // 1
            RGBA12::GRAY,        // 2
            RGBA12::WHITE,       // 3
            RGBA12::DARK_RED,    // 4
            RGBA12::RED,         // 5
            RGBA12::LIGHT_RED,   // 6
            RGBA12::ORANGE,      // 7
            RGBA12::YELLOW,      // 8
            RGBA12::DARK_GREEN,  // 9
            RGBA12::GREEN,       // 10
            RGBA12::LIGHT_GREEN, // 11
            RGBA12::DARK_BLUE,   // 12
            RGBA12::BLUE,        // 13
            RGBA12::LIGHT_BLUE,  // 14
            RGBA12::PINK,        // 15
        ];
        self.palette_head = 16;
    }

    pub fn set_color(&mut self, id: ColorID, color: RGBA12) {
        assert!(id.0 < COLORS_PER_PALETTE as u8, "Invalid color ID");
        self.palette[id.0 as usize] = color;
    }

    // pub fn palette_cycle(&mut self, color_mapping: u8, start_index: u8, end_index: u8, delta: i8) {
    //     let original_colors = self.mapping[color_mapping as usize];

    //     for index in start_index as isize..=end_index as isize {
    //         let mut new_index = index + delta as isize;
    //         if delta > 0 {
    //             if new_index > end_index as isize {
    //                 new_index = start_index as isize;
    //             }
    //         } else {
    //             if new_index < start_index as isize {
    //                 new_index = end_index as isize;
    //             }
    //         }
    //         let color = &mut self.mapping[color_mapping as usize][index as usize];
    //         *color = original_colors[new_index as usize];
    //     }
    // }

    /// Adds unique colors to the bank, and returns a palette remap, if any
    pub fn append(&mut self, colors: &[RGBA12]) -> Result<PaletteRemap, &'static str> {
        let mut color_remap = [0u8; COLORS_PER_PALETTE as usize];

        for src_color_idx in 0..colors.len() {
            let src_color = colors[src_color_idx];

            // Check if this color already exists in dest palette
            let mut found_idx = None;
            for dest_color_idx in 0..self.palette_head as usize {
                if self.palette[dest_color_idx] == src_color {
                    found_idx = Some(dest_color_idx);
                    break;
                }
            }

            if let Some(existing_idx) = found_idx {
                // Color already exists, reuse it
                color_remap[src_color_idx] = existing_idx as u8;
            } else {
                // New color, add it
                if self.palette_head >= COLORS_PER_PALETTE as u8 {
                    return Err("Not enough space in bank for colors");
                }
                color_remap[src_color_idx] = self.palette_head;
                self.palette[self.palette_head as usize] = src_color;
                self.palette_head += 1;
            }
        }
        Ok(color_remap)
    }
}
