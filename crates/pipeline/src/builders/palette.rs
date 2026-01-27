use crate::*;
use std::collections::HashMap;
use tato_video::*;

/// Collects colors from images and generates palette code.
#[derive(Debug, Clone)]
pub struct PaletteBuilder {
    /// Palette identifier used in generated code.
    pub name: String,
    /// Collected 12-bit RGBA colors.
    pub rgb_colors: Vec<RGBA12>,
    /// Maps colors to their palette indices.
    pub rgb_to_index: HashMap<RGBA12, u8>,
    id: u8,
}

impl PaletteBuilder {
    /// Creates a new palette builder with the given name.
    pub fn new(name: &str) -> Self {
        crate::ensure_init_build();
        PaletteBuilder {
            name: String::from(name),
            rgb_colors: vec![],
            rgb_to_index: HashMap::new(),
            id: 0, // ID no longer used in new API
        }
    }

    /// Adds a color to the palette. Panics if palette is full.
    pub fn push(&mut self, color: RGBA12) {
        if self.rgb_colors.len() == COLORS_PER_PALETTE as usize {
            panic!("Palette error: capacity of {} exceeded.", COLORS_PER_PALETTE)
        }
        self.rgb_colors.push(color);
    }

    /// Returns the palette ID.
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Writes palette constants to a file relative to export path. Skipped if empty.
    pub fn write(&self, file_path: &str) {
        // Check if there are any colors to write
        if self.rgb_colors.is_empty() {
            // No colors to write, skip file generation
            return;
        }

        // Make file_path relative to export path
        let settings = crate::get_build_settings();
        let full_path = std::path::Path::new(&settings.asset_export_path)
            .join(file_path)
            .to_str()
            .expect("Could not convert path to string")
            .to_string();

        let mut code = CodeWriter::new(&full_path);

        // Write header (palette doesn't use crate assets or allow_unused)
        code.write_header(false, false, true);

        // Write palette colors
        code.write_color_array(&self.name, &self.rgb_colors);

        // Format the output
        code.format_output(&full_path);

        // Register this file for mod.rs generation
        crate::register_generated_file(&full_path);
    }
}
