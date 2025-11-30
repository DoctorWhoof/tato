use std::collections::HashSet;

/// Maximum number of colors per sub-palette
const COLORS_PER_TILE: usize = 4;

/// Builder for managing sub-palette colors with merging capabilities.
/// Two palettes can merge if their combined unique colors don't exceed COLORS_PER_TILE.
#[derive(Debug, Clone, PartialEq)]
pub struct SubPaletteBuilder {
    /// Colors in the sub-palette, always kept sorted for consistent comparison
    colors: Vec<u8>,
}

impl SubPaletteBuilder {
    /// Creates a new empty SubPaletteBuilder.
    pub fn new() -> Self {
        Self {
            colors: Vec::new(),
        }
    }

    /// Creates a SubPaletteBuilder from colors. Removes duplicates and sorts.
    pub fn from_colors(colors: &[u8]) -> Self {
        if colors.is_empty() {
            return Self {
                colors: vec![],
            };
        }
        
        let mut unique_colors: Vec<u8> = colors.iter().cloned().collect::<HashSet<_>>().into_iter().collect();
        unique_colors.sort_unstable();
        
        if unique_colors.len() > COLORS_PER_TILE {
            panic!("Cannot create SubPaletteBuilder with {} colors (max {}). Colors: {:?}", 
                unique_colors.len(), 
                COLORS_PER_TILE,
                unique_colors
            );
        }
        
        Self {
            colors: unique_colors,
        }
    }

    /// Creates a SubPaletteBuilder from a 4-element array. Filters padding zeros.
    pub fn from_array(array: [u8; COLORS_PER_TILE]) -> Self {
        // In sub-palette arrays, trailing zeros are always padding
        // Find the actual color count by looking for the last non-zero
        let mut actual_colors = Vec::new();
        
        for &color in &array {
            if color == 0 && actual_colors.is_empty() {
                // Leading zero - this is a real color
                actual_colors.push(color);
            } else if color != 0 {
                // Non-zero color
                actual_colors.push(color);
            }
            // Trailing zeros are ignored as padding
        }
        
        // If no colors found, default to single zero
        if actual_colors.is_empty() {
            actual_colors.push(0);
        }
        
        // Remove duplicates and sort
        let unique_colors: Vec<u8> = actual_colors.into_iter().collect::<HashSet<_>>().into_iter().collect();
        let mut sorted_colors = unique_colors;
        sorted_colors.sort_unstable();
        
        Self {
            colors: sorted_colors,
        }
    }

    /// Returns the colors in this sub-palette in sorted order.
    pub fn colors(&self) -> &[u8] {
        &self.colors
    }

    /// Returns the number of colors currently in this sub-palette.
    pub fn color_count(&self) -> usize {
        self.colors.len()
    }

    /// Returns the number of available color slots remaining.
    pub fn available_slots(&self) -> usize {
        COLORS_PER_TILE - self.colors.len()
    }

    /// Checks if this sub-palette is full.
    pub fn is_full(&self) -> bool {
        self.colors.len() >= COLORS_PER_TILE
    }

    /// Checks if this sub-palette contains all the specified colors.
    pub fn contains_all(&self, colors: &[u8]) -> bool {
        let color_set: HashSet<u8> = self.colors.iter().cloned().collect();
        colors.iter().all(|&color| color_set.contains(&color))
    }

    /// Checks if this sub-palette can merge with another.
    pub fn can_merge(&self, other: &SubPaletteBuilder) -> bool {
        if self.colors.len() > COLORS_PER_TILE || other.colors.len() > COLORS_PER_TILE {
            eprintln!("WARNING: Invalid palette sizes - self: {}, other: {}", self.colors.len(), other.colors.len());
            return false;
        }
        
        // Collect all unique colors from both palettes
        let mut all_colors: HashSet<u8> = HashSet::new();
        all_colors.extend(&self.colors);
        all_colors.extend(&other.colors);

        all_colors.len() <= COLORS_PER_TILE
    }

    /// Merges this sub-palette with another, consuming both. Returns None if merge is not possible.
    pub fn merge(self, other: SubPaletteBuilder) -> Option<SubPaletteBuilder> {
        if !self.can_merge(&other) {
            return None;
        }

        // Combine and deduplicate colors
        let mut all_colors: HashSet<u8> = HashSet::new();
        all_colors.extend(self.colors);
        all_colors.extend(other.colors);

        let mut merged_colors: Vec<u8> = all_colors.into_iter().collect();
        merged_colors.sort_unstable();

        Some(SubPaletteBuilder {
            colors: merged_colors,
        })
    }

    /// Tries to add a color. Returns true if successful, false if would exceed capacity.
    pub fn try_add_color(&mut self, color: u8) -> bool {
        if self.colors.contains(&color) {
            return true; // Color already exists
        }

        if self.is_full() {
            return false; // No space
        }

        self.colors.push(color);
        self.colors.sort_unstable();
        true
    }

    /// Converts to a 4-element array. Unused slots are filled with zeros.
    pub fn to_array(&self) -> [u8; COLORS_PER_TILE] {
        if self.colors.len() > COLORS_PER_TILE {
            panic!("SubPaletteBuilder has {} colors but max is {}", self.colors.len(), COLORS_PER_TILE);
        }
        
        let mut array = [0u8; COLORS_PER_TILE];
        for (i, &color) in self.colors.iter().enumerate() {
            if i < COLORS_PER_TILE {
                array[i] = color;
            } else {
                eprintln!("WARNING: Skipping color at index {} (exceeds COLORS_PER_TILE)", i);
            }
        }
        array
    }

    /// Creates a mapping from colors to indices in this sub-palette. Returns None if any color is not found.
    pub fn create_remapping(&self, colors: &[u8]) -> Option<Vec<u8>> {
        let mut remapping = Vec::with_capacity(colors.len());
        
        for &color in colors {
            match self.colors.iter().position(|&c| c == color) {
                Some(index) => {
                    if index >= COLORS_PER_TILE {
                        eprintln!("WARNING: Color index {} exceeds COLORS_PER_TILE ({})", index, COLORS_PER_TILE);
                        return None;
                    }
                    remapping.push(index as u8);
                },
                None => {
                    eprintln!("WARNING: Color {} not found in sub-palette {:?}", color, self.colors);
                    return None; // Color not found in this sub-palette
                }
            }
        }
        
        if remapping.len() != colors.len() {
            eprintln!("WARNING: Remapping length mismatch: expected {}, got {}", colors.len(), remapping.len());
            return None;
        }
        
        Some(remapping)
    }
}

impl Default for SubPaletteBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let builder = SubPaletteBuilder::new();
        assert_eq!(builder.color_count(), 0);
        assert_eq!(builder.available_slots(), 4);
    }

    #[test]
    fn test_from_colors() {
        let builder = SubPaletteBuilder::from_colors(&[2, 1, 3, 1]); // Duplicates should be removed
        assert_eq!(builder.colors(), &[1, 2, 3]); // Should be sorted
        assert_eq!(builder.color_count(), 3);
    }

    #[test]
    fn test_from_array() {
        let builder = SubPaletteBuilder::from_array([1, 3, 2, 0]);
        assert_eq!(builder.colors(), &[1, 2, 3]); // Zeros filtered out and sorted
    }

    #[test]
    fn test_can_merge_same_colors() {
        let builder1 = SubPaletteBuilder::from_colors(&[0, 1, 2, 3]);
        let builder2 = SubPaletteBuilder::from_colors(&[0, 1, 2, 3]);
        assert!(builder1.can_merge(&builder2));
    }

    #[test]
    fn test_can_merge_compatible() {
        let builder1 = SubPaletteBuilder::from_colors(&[0]);
        let builder2 = SubPaletteBuilder::from_colors(&[0, 1, 2, 3]);
        assert!(builder1.can_merge(&builder2));
        
        let builder3 = SubPaletteBuilder::from_colors(&[0, 2, 4]);
        let builder4 = SubPaletteBuilder::from_colors(&[0, 2, 5]);
        assert!(builder3.can_merge(&builder4));
    }

    #[test]
    fn test_cannot_merge_incompatible() {
        let builder1 = SubPaletteBuilder::from_colors(&[0, 2, 4, 5]);
        let builder2 = SubPaletteBuilder::from_colors(&[0, 2, 5, 6]);
        assert!(!builder1.can_merge(&builder2)); // Would need 5 colors total
    }

    #[test]
    fn test_merge() {
        let builder1 = SubPaletteBuilder::from_colors(&[0, 2, 4]);
        let builder2 = SubPaletteBuilder::from_colors(&[0, 2, 5]);
        
        let merged = builder1.merge(builder2).unwrap();
        assert_eq!(merged.colors(), &[0, 2, 4, 5]);
    }

    #[test]
    fn test_create_remapping() {
        let builder = SubPaletteBuilder::from_colors(&[0, 2, 4, 5]);
        let remapping = builder.create_remapping(&[2, 0, 4, 2]).unwrap();
        assert_eq!(remapping, vec![1, 0, 2, 1]); // Indices in the sorted color array
    }

    #[test]
    fn test_to_array() {
        let builder = SubPaletteBuilder::from_colors(&[5, 2, 8]);
        assert_eq!(builder.to_array(), [2, 5, 8, 0]); // Sorted with padding zeros
    }

    #[test]
    fn test_zero_color_handling() {
        // Test with actual zero color
        let builder = SubPaletteBuilder::from_colors(&[0]);
        assert_eq!(builder.colors(), &[0]);
        assert_eq!(builder.to_array(), [0, 0, 0, 0]);
        
        // Test zero color mixed with others
        let builder = SubPaletteBuilder::from_colors(&[0, 1, 2]);
        assert_eq!(builder.colors(), &[0, 1, 2]);
        assert_eq!(builder.to_array(), [0, 1, 2, 0]);
    }

    #[test]
    fn test_from_array_zero_handling() {
        // All zeros should result in single zero color
        let builder = SubPaletteBuilder::from_array([0, 0, 0, 0]);
        assert_eq!(builder.colors(), &[0]);
        
        // Mixed zeros and colors
        let builder = SubPaletteBuilder::from_array([1, 0, 2, 0]);
        assert_eq!(builder.colors(), &[1, 2]); // Padding zeros filtered out
        
        // Zero as first color with others - first zero is real color
        let builder = SubPaletteBuilder::from_array([0, 1, 2, 3]);
        assert_eq!(builder.colors(), &[0, 1, 2, 3]); // Zero kept as actual color
    }

    #[test]
    fn test_remapping_with_duplicates() {
        let builder = SubPaletteBuilder::from_colors(&[1, 3, 5]);
        
        // Test remapping with duplicate colors in input
        let remapping = builder.create_remapping(&[3, 1, 3, 5]).unwrap();
        assert_eq!(remapping, vec![1, 0, 1, 2]); // 3->1, 1->0, 3->1, 5->2
        
        // Test remapping with missing color
        assert!(builder.create_remapping(&[1, 2, 3]).is_none()); // 2 is missing
    }

    #[test]
    fn test_edge_case_merges() {
        // Empty with full palette
        let empty = SubPaletteBuilder::new();
        let full = SubPaletteBuilder::from_colors(&[0, 1, 2, 3]);
        assert!(empty.can_merge(&full));
        
        let merged = empty.merge(full).unwrap();
        assert_eq!(merged.colors(), &[0, 1, 2, 3]);
        
        // Two full palettes with overlap
        let full1 = SubPaletteBuilder::from_colors(&[0, 1, 2, 3]);
        let full2 = SubPaletteBuilder::from_colors(&[2, 3, 4, 5]);
        assert!(!full1.can_merge(&full2)); // Would need 6 colors
    }

    #[test]
    fn test_array_roundtrip() {
        let original = [1, 3, 0, 2];
        let builder = SubPaletteBuilder::from_array(original);
        let result = builder.to_array();
        
        // Should be sorted with zeros as padding
        assert_eq!(result, [1, 2, 3, 0]);
        
        // Test with all unique colors
        let original = [3, 1, 4, 2];
        let builder = SubPaletteBuilder::from_array(original);
        let result = builder.to_array();
        assert_eq!(result, [1, 2, 3, 4]);
    }
}