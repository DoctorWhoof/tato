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
        let mut unique_colors: Vec<u8> = colors.iter().cloned().collect::<HashSet<_>>().into_iter().collect();
        unique_colors.sort_unstable();
        
        assert!(unique_colors.len() <= COLORS_PER_TILE, 
            "Cannot create SubPaletteBuilder with {} colors (max {})", 
            unique_colors.len(), 
            COLORS_PER_TILE
        );
        
        Self {
            colors: unique_colors,
        }
    }

    /// Creates a SubPaletteBuilder from a 4-element array. Filters padding zeros.
    pub fn from_array(array: [u8; COLORS_PER_TILE]) -> Self {
        // Filter out zeros except when it's the only color or the first color
        let mut colors = Vec::new();
        for (i, &color) in array.iter().enumerate() {
            if color != 0 || (i == 0 && array.iter().all(|&c| c == 0 || c == color)) {
                colors.push(color);
            }
        }
        
        // Remove duplicates and sort
        let unique_colors: Vec<u8> = colors.into_iter().collect::<HashSet<_>>().into_iter().collect();
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
        // Collect all unique colors from both palettes
        let mut all_colors: HashSet<u8> = HashSet::new();
        all_colors.extend(&self.colors);
        all_colors.extend(&other.colors);

        // Can merge if the total unique colors don't exceed the limit
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
        let mut array = [0u8; COLORS_PER_TILE];
        for (i, &color) in self.colors.iter().enumerate() {
            if i < COLORS_PER_TILE {
                array[i] = color;
            }
        }
        array
    }

    /// Creates a mapping from colors to indices in this sub-palette. Returns None if any color is not found.
    pub fn create_remapping(&self, colors: &[u8]) -> Option<Vec<u8>> {
        let mut remapping = Vec::with_capacity(colors.len());
        
        for &color in colors {
            match self.colors.iter().position(|&c| c == color) {
                Some(index) => remapping.push(index as u8),
                None => return None, // Color not found in this sub-palette
            }
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
}