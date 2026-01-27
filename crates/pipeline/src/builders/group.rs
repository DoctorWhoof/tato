//! Tile group builder for categorizing tiles.

use crate::{CodeWriter, Pixels};
use std::collections::HashMap;

/// Manages tile groups for collision or rendering categories.
#[derive(Debug, Default)]
pub struct GroupBuilder {
    /// Maps tile pixels to their group bitmask.
    pub hash: HashMap<Pixels, u8>,
    /// Group names indexed by group index (0-based).
    pub names: Vec<String>,
}

impl GroupBuilder {
    /// Creates an empty group builder.
    pub fn new() -> Self {
        crate::ensure_init_build();
        Self::default()
    }

    /// Adds a named group, returns its 1-based index. Returns existing index if name exists.
    pub fn add_group(&mut self, name: &str) -> u8 {
        // Check if group name already exists
        for (index, existing_name) in self.names.iter().enumerate() {
            if existing_name == name {
                return (index + 1) as u8; // Convert 0-based back to 1-based
            }
        }

        // Group doesn't exist, create new one
        let group_index = self.names.len() + 1;
        assert!(group_index > 0 && group_index <= 16, "Group index must be between 1-16");
        let group_index = group_index as u8;

        let vec_index = (group_index - 1) as usize; // Convert 1-based to 0-based
        if self.names.len() <= vec_index {
            self.names.resize(vec_index + 1, String::new());
        }
        self.names[vec_index] = String::from(name);

        group_index
    }

    /// Registers a tile as belonging to a group (internal use).
    pub(crate) fn register_tile(&mut self, canonical_tile: Pixels, group_index: u8) {
        let group_bit = 1u8 << (group_index - 1); // Convert 1-based index to bit position
        let current_groups = self.hash.get(&canonical_tile).unwrap_or(&0);
        self.hash.insert(canonical_tile, current_groups | group_bit);
    }

    /// Writes group constants to a file. Skipped if empty.
    pub fn write(&self, file_path: &str) {
        // Check if there are any non-empty groups
        let has_groups = self.names.iter().any(|name| !name.is_empty());
        
        if !has_groups {
            // No groups to write, skip file generation
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

        // Write group constants
        for (index, name) in self.names.iter().enumerate() {
            if !name.is_empty() {
                let group_index = (index + 1) as u8; // Convert 0-based back to 1-based
                code.write_group_constant(name, group_index);
            }
        }

        // Format the output
        code.format_output(&full_path);

        // Register this file for mod.rs generation
        crate::register_generated_file(&full_path);
    }
}
