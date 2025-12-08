use crate::{CanonicalTile, CodeWriter, Pixels};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct GroupBuilder {
    pub hash: HashMap<Pixels, u8>, // Key: tile, value:group bits
    pub names: Vec<String>,        // Index is group index (0-based), value is group name
}

impl GroupBuilder {
    pub fn new() -> Self {
        crate::ensure_init_build();
        Self::default()
    }

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

    pub(crate) fn register_tile(&mut self, canonical_tile: Pixels, group_index: u8) {
        let group_bit = 1u8 << (group_index - 1); // Convert 1-based index to bit position
        let current_groups = self.hash.get(&canonical_tile).unwrap_or(&0);
        self.hash.insert(canonical_tile, current_groups | group_bit);
    }

    /// Writes the group constants to a file
    pub fn write(&self, file_path: &str) {
        let mut code = CodeWriter::new(file_path);

        // Write group constants
        if !self.names.is_empty() {
            for (index, name) in self.names.iter().enumerate() {
                if !name.is_empty() {
                    let group_index = (index + 1) as u8; // Convert 0-based back to 1-based
                    code.write_group_constant(name, group_index);
                }
            }
        }

        // Format the output
        code.format_output(file_path);
    }
}
