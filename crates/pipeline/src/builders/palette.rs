// use crate::Color14Bit;
use crate::*;
use std::collections::HashMap;
use tato_video::*;

// #[derive(Debug, Clone, Copy)]
// pub struct PaletteID(pub u8);

#[derive(Debug, Clone)]
pub struct PaletteBuilder {
    pub name: String,
    pub colors: Vec<RGBA12>,
    pub color_hash: HashMap<RGBA12, u8>,
    id: u8,
}

impl PaletteBuilder {
    pub fn new(name: &str) -> Self {
        crate::ensure_init_build();
        PaletteBuilder {
            name: String::from(name),
            colors: vec![],
            color_hash: HashMap::new(),
            id: 0, // ID no longer used in new API
        }
    }

    pub fn push(&mut self, color: RGBA12) {
        if self.colors.len() == COLORS_PER_PALETTE as usize {
            panic!("Palette error: capacity of {} exceeded.", COLORS_PER_PALETTE)
        }
        self.colors.push(color);
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
        code.write_color_array(&self.name, &self.colors);

        // Format the output
        code.format_output(file_path);
    }
}
