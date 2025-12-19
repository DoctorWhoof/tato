use crate::*;
use std::collections::HashMap;
use tato_video::*;

#[derive(Debug, Clone)]
pub struct PaletteBuilder {
    pub name: String,
    pub rgb_colors: Vec<RGBA12>,
    pub rgb_to_index: HashMap<RGBA12, u8>,
    id: u8,
}

impl PaletteBuilder {
    pub fn new(name: &str) -> Self {
        crate::ensure_init_build();
        PaletteBuilder {
            name: String::from(name),
            rgb_colors: vec![],
            rgb_to_index: HashMap::new(),
            id: 0, // ID no longer used in new API
        }
    }

    pub fn push(&mut self, color: RGBA12) {
        if self.rgb_colors.len() == COLORS_PER_PALETTE as usize {
            panic!("Palette error: capacity of {} exceeded.", COLORS_PER_PALETTE)
        }
        self.rgb_colors.push(color);
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    /// Writes the palette constants to a file
    pub fn write(&self, file_path: &str) {
        let mut code = CodeWriter::new(file_path);

        // Write header (palette doesn't use crate assets or allow_unused)
        code.write_header(false, false);

        // Write palette colors
        code.write_color_array(&self.name, &self.rgb_colors);

        // Format the output
        code.format_output(file_path);
    }
}
