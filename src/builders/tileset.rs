use core::array::from_fn;
use tato_video::*;

use super::*;
use crate::*;
use std::collections::{HashMap, HashSet};
use std::{vec, vec::Vec};

const TILE_SIZE_BYTES: usize = TILE_SIZE as usize * TILE_SIZE as usize;
type TileData = [u8; TILE_SIZE_BYTES]; // 64 bytes, stack allocated

#[derive(Debug, Clone, Copy)]
pub struct TilesetBuilderID(pub u8);

pub struct TilesetBuilder {
    pub allow_tile_transforms: bool,
    pub name: String,
    pub pixels: Vec<u8>,
    // Stores the Tile (ID and Flags) for each unique set of pixels
    pub tile_hash: HashMap<Vec<u8>, Cell>,
    // Stores the names of each unique palette
    pub sub_palette_name_hash: HashMap<[u8; SUB_PALETTE_COLOR_COUNT], String>,
    // The actual color indices for the sub-palettes
    pub sub_palettes: Vec<[u8; SUB_PALETTE_COLOR_COUNT]>,
    pub anims: Vec<AnimBuilder>,
    pub maps: Vec<MapBuilder>,
    pub single_tiles: Vec<SingleTileBuilder>,
    pub palette_id: PaletteID,
    next_tile: u8,
    sub_palette_head: usize,
    color_set_to_palette: HashMap<String, u8>,
}

impl TilesetBuilder {
    pub fn new(name: String, palette_id: PaletteID) -> Self {
        Self {
            allow_tile_transforms: true,
            name,
            pixels: vec![],
            tile_hash: HashMap::new(),
            sub_palette_name_hash: HashMap::new(),
            next_tile: 0,
            anims: vec![],
            maps: vec![],
            single_tiles: vec![],
            palette_id,
            sub_palettes: Vec::new(),
            sub_palette_head: 0,
            color_set_to_palette: HashMap::new(),
        }
    }

    fn push_sub_palette(&mut self, sub_palette: &[u8]) -> u8 {
        if self.sub_palette_head == SUB_PALETTE_COUNT {
            panic!("Tileset error: capacity of {} sub-palettes exceeded.", self.sub_palettes.len())
        }

        self.sub_palettes.push(from_fn(|i| match sub_palette.get(i) {
            Some(value) => *value,
            None => 0,
        }));

        let result = u8::try_from(self.sub_palette_head).unwrap();
        self.sub_palette_head += 1;
        result
    }

    /// Main workhorse function! Splits images into 8x8 tiles (8 pixels wide, as many pixels as it needs tall)
    /// grouped by frame. Returns the indices, the flags and the number of tiles
    pub fn add_tiles(&mut self, img: &PalettizedImg, palette: &PaletteBuilder) -> Vec<Cell> {
        // Phase 1: Collect all unique color sets without creating sub-palettes yet
        let color_sets = self.collect_tile_color_sets(img);

        // Phase 2: Allocate optimal sub-palettes
        self.allocate_sub_palettes(&color_sets, palette);

        // Phase 3: Process tiles with pre-allocated sub-palettes
        self.process_tiles_with_palettes(img, palette)
    }

    // Helper method to collect all unique color sets from an image
    fn collect_tile_color_sets(&self, img: &PalettizedImg) -> Vec<HashSet<u8>> {
        let mut unique_color_sets = Vec::new();

        for frame_v in 0..img.frames_v as usize {
            for frame_h in 0..img.frames_h as usize {
                for row in 0..img.rows_per_frame as usize {
                    for col in 0..img.cols_per_frame as usize {
                        let abs_col = (frame_h * img.cols_per_frame as usize) + col;
                        let abs_row = (frame_v * img.rows_per_frame as usize) + row;

                        let mut colors = HashSet::new();

                        // Collect colors for this tile
                        for y in 0..TILE_SIZE as usize {
                            for x in 0..TILE_SIZE as usize {
                                let abs_x = (TILE_SIZE as usize * abs_col) + x;
                                let abs_y = (TILE_SIZE as usize * abs_row) + y;
                                let index = (img.width * abs_y) + abs_x;
                                let value = img.pixels[index];

                                // if value != 0 {
                                // Typically 0 is transparent/background
                                colors.insert(value);
                                // }
                            }
                        }

                        // Check if this color set is already accounted for
                        if !colors.is_empty() && !unique_color_sets.contains(&colors) {
                            if colors.len() > SUB_PALETTE_COLOR_COUNT {
                                panic!(
                                    "Tile at position ({}, {}) exceeds the {} color limit",
                                    abs_col, abs_row, SUB_PALETTE_COLOR_COUNT
                                );
                            }
                            unique_color_sets.push(colors);
                        }
                    }
                }
            }
        }

        unique_color_sets
    }

    // Helper method to allocate optimal sub-palettes
    fn allocate_sub_palettes(&mut self, color_sets: &[HashSet<u8>], palette: &PaletteBuilder) {
        // Clear existing data
        // self.sub_palette_name_hash.clear();
        // self.sub_palettes.clear();
        // self.sub_palette_head = 0;

        // Sort color sets by size (largest first)
        let mut sorted_sets = color_sets.to_vec();
        sorted_sets.sort_by(|a, b| b.len().cmp(&a.len()));

        // Map to track which sub-palette each color set is assigned to
        let mut color_set_to_palette = HashMap::new();

        for color_set in sorted_sets {
            // Create the string key BEFORE any operations that could move color_set
            let color_set_key = color_set_to_string(&color_set);
            let mut assigned = false;

            // Try to find an existing palette this set fits within
            for (i, sub_pal) in self.sub_palettes.iter().enumerate() {
                let mut combined_palette = HashSet::new();
                for &color in sub_pal {
                    // if color != 0 {
                    combined_palette.insert(color);
                    // }
                }

                // Check if combining would stay within limit
                let mut would_fit = true;
                for &color in &color_set {
                    if !combined_palette.contains(&color) {
                        combined_palette.insert(color);
                        if combined_palette.len() > SUB_PALETTE_COLOR_COUNT {
                            would_fit = false;
                            break;
                        }
                    }
                }

                if would_fit {
                    // Update the sub-palette to include these colors
                    let palette_id = i as u8;

                    // Convert to sorted array
                    let mut combined_vec: Vec<_> = combined_palette.into_iter().collect();
                    combined_vec.sort();

                    let palette_array = create_palette_array(&combined_vec);

                    self.sub_palettes[i] = palette_array;

                    // Store mapping using our saved key
                    color_set_to_palette.insert(color_set_key.clone(), palette_id);

                    // Update hash map for name lookup
                    let name = format!("{}_{}", palette.name, palette_id);
                    self.sub_palette_name_hash.insert(palette_array, name);

                    assigned = true;
                    break;
                }
            }

            // If couldn't fit in any existing palette, create a new one
            if !assigned {
                if self.sub_palette_head == SUB_PALETTE_COUNT {
                    panic!(
                        "Tileset error: capacity of {} sub-palettes exceeded.",
                        SUB_PALETTE_COUNT
                    );
                }

                // Convert set to sorted array
                let mut color_vec: Vec<_> = color_set.into_iter().collect();
                color_vec.sort();

                let palette_array = create_palette_array(&color_vec);

                let palette_id = self.push_sub_palette(&palette_array);

                // Store mapping using our saved key
                color_set_to_palette.insert(color_set_key, palette_id);

                // Set name
                let name = format!("{}_{}", palette.name, palette_id);
                self.sub_palette_name_hash.insert(palette_array, name);
            }
        }

        // Store the mapping for use in the processing phase
        self.color_set_to_palette = color_set_to_palette;
    }
    // Phase 3: Process tiles with pre-allocated sub-palettes
    fn process_tiles_with_palettes(
        &mut self,
        img: &PalettizedImg,
        palette: &PaletteBuilder,
    ) -> Vec<Cell> {
        let mut tiles = vec![];
        let tile_length = TILE_SIZE as usize * TILE_SIZE as usize;

        for frame_v in 0..img.frames_v as usize {
            for frame_h in 0..img.frames_h as usize {
                for row in 0..img.rows_per_frame as usize {
                    for col in 0..img.cols_per_frame as usize {
                        let abs_col = (frame_h * img.cols_per_frame as usize) + col;
                        let abs_row = (frame_v * img.rows_per_frame as usize) + row;

                        let mut tile_candidate_original = vec![0u8; tile_length];
                        let mut color_set = HashSet::new();

                        // Extract tile pixels and collect colors (original indices)
                        for y in 0..TILE_SIZE as usize {
                            for x in 0..TILE_SIZE as usize {
                                let abs_x = (TILE_SIZE as usize * abs_col) + x;
                                let abs_y = (TILE_SIZE as usize * abs_row) + y;
                                let index = (img.width * abs_y) + abs_x;
                                let value = img.pixels[index];

                                tile_candidate_original[(TILE_SIZE as usize * y) + x] = value;
                                color_set.insert(value);
                            }
                        }

                        // Find the sub-palette for this color set
                        let color_set_key = color_set_to_string(&color_set);
                        let sub_palette_id =
                            self.color_set_to_palette.get(&color_set_key).copied().unwrap_or_else(
                                || self.find_or_create_sub_palette(&color_set, palette),
                            );

                        // Get the sub-palette
                        let sub_palette = &self.sub_palettes[sub_palette_id as usize];

                        // Convert original indices to sub-palette relative indices
                        let tile_normalized =
                            normalize_tile_to_sub_palette(&tile_candidate_original, sub_palette);

                        // Try to find this tile in any transformation
                        if let Some((existing_tile, transform_flags)) =
                            self.find_existing_transformation(&tile_normalized)
                        {
                            // Create a new cell with the existing tile ID but fresh flags
                            let mut reused_tile = Cell {
                                id: existing_tile.id,
                                flags: transform_flags, // Use the transformation flags directly
                            };

                            // Set the correct palette for this usage
                            reused_tile.flags.set_palette(PaletteID(sub_palette_id));

                            tiles.push(reused_tile);
                        } else {
                            // Create new tile and add all transformations to hash
                            let mut new_tile =
                                Cell { id: TileID(self.next_tile), flags: TileFlags::default() };
                            new_tile.flags.set_palette(PaletteID(sub_palette_id));

                            // Store original tile in hash
                            self.tile_hash.insert(tile_normalized.clone(), new_tile);

                            // Generate and store all transformations if enabled
                            if self.allow_tile_transforms {
                                self.add_tile_transformations(&tile_normalized, new_tile);
                            }

                            // Store normalized tile data in pixels (only the original)
                            self.pixels.extend_from_slice(&tile_normalized);

                            tiles.push(new_tile);
                            self.next_tile += 1;
                        }
                    }
                }
            }
        }

        tiles
    }

    // Helper to find if any transformation of this tile already exists
    fn find_existing_transformation(&self, tile: &[u8]) -> Option<(Cell, TileFlags)> {
        // Check original tile first
        if let Some(existing) = self.tile_hash.get(tile) {
            return Some((*existing, existing.flags));
        }

        if !self.allow_tile_transforms {
            return None;
        }

        // Try all 8 possible transformations
        for flip_x in [false, true] {
            for flip_y in [false, true] {
                for rotation in [false, true] {
                    // Skip the original (already checked above)
                    if !flip_x && !flip_y && !rotation {
                        continue;
                    }

                    // Apply transformation to current tile
                    let transformed_tile = transform_tile(tile, flip_x, flip_y, rotation);

                    // Check if this transformation exists in our hash
                    if let Some(existing) = self.tile_hash.get(&transformed_tile) {
                        return Some((*existing, existing.flags));
                    }
                }
            }
        }

        None
    }

    // Helper to add all transformations of a tile to the hash map
    fn add_tile_transformations(&mut self, tile: &[u8], base_tile: Cell) {
        // Create a base tile with clean flags for storing transformations
        let storage_tile = Cell {
            id: base_tile.id,
            flags: TileFlags::default(), // Clean slate
        };

        // Generate all 8 possible combinations of the 3 transform bits
        for flip_x in [false, true] {
            for flip_y in [false, true] {
                for rotation in [false, true] {
                    // Skip the original combination (0,0,0) as it's already stored
                    if !flip_x && !flip_y && !rotation {
                        continue;
                    }

                    // Apply the transformation to the tile pixels
                    let transformed_tile = transform_tile(tile, flip_x, flip_y, rotation);

                    // Create the flags for this transformation
                    let mut tile_with_flags = storage_tile;
                    tile_with_flags.flags.set_flip_x(flip_x);
                    tile_with_flags.flags.set_flip_y(flip_y);
                    tile_with_flags.flags.set_rotation(rotation);

                    // Store in hash map
                    self.tile_hash.insert(transformed_tile, tile_with_flags);
                }
            }
        }
    }

    fn find_or_create_sub_palette(
        &mut self,
        color_set: &HashSet<u8>,
        palette: &PaletteBuilder,
    ) -> u8 {
        // Try to find an existing sub-palette that contains all these colors
        for (i, sub_pal) in self.sub_palettes.iter().enumerate() {
            let mut pal_colors = HashSet::new();
            for &color in sub_pal {
                if color != 0 {
                    // Assuming 0 is padding/transparent
                    pal_colors.insert(color);
                }
            }

            if color_set.is_subset(&pal_colors) {
                return i as u8;
            }
        }

        // If no existing palette works, create a new one
        let mut color_vec: Vec<_> = color_set.iter().cloned().collect();
        color_vec.sort();
        let palette_array = create_palette_array(&color_vec);

        let palette_id = self.push_sub_palette(&palette_array);
        let name = format!("{}_{}", palette.name, palette_id);
        self.sub_palette_name_hash.insert(palette_array, name);

        palette_id
    }
}

// Helper to convert a set to a sorted array for consistent storage
fn create_palette_array(color_vec: &[u8]) -> [u8; SUB_PALETTE_COLOR_COUNT] {
    from_fn(|i| if i < color_vec.len() { color_vec[i] } else { 0 })
}

// Helper to create a consistent string key from a color set
fn color_set_to_string(color_set: &HashSet<u8>) -> String {
    let mut colors: Vec<_> = color_set.iter().cloned().collect();
    colors.sort();
    colors.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("_")
}

// Helper function to convert original palette indices to sub-palette relative indices
fn normalize_tile_to_sub_palette(
    original_tile: &[u8],
    sub_palette: &[u8; SUB_PALETTE_COLOR_COUNT],
) -> Vec<u8> {
    original_tile
        .iter()
        .map(|&original_color| {
            // Find this color's position in the sub-palette
            sub_palette.iter().position(|&pal_color| pal_color == original_color).unwrap_or(0) as u8 // Default to 0 if not found
        })
        .collect()
}

// Apply transformation based on the three TileFlags bits
fn transform_tile(tile: &[u8], flip_x: bool, flip_y: bool, rotation: bool) -> Vec<u8> {
    let size = TILE_SIZE as usize;
    let mut result = vec![0u8; tile.len()];

    for y in 0..size {
        for x in 0..size {
            let src_idx = y * size + x;

            // Start with original coordinates
            let mut dst_x = x;
            let mut dst_y = y;

            // Apply rotation first (this is the mystery transformation)
            if rotation {
                // Based on the TileFlags convention, rotation seems to be:
                // a 90° counter-clockwise rotation (since rotate_left() sets this bit)
                let temp_x = dst_x;
                dst_x = dst_y;
                dst_y = size - 1 - temp_x;
            }

            // Apply horizontal flip
            if flip_x {
                dst_x = size - 1 - dst_x;
            }

            // Apply vertical flip
            if flip_y {
                dst_y = size - 1 - dst_y;
            }

            let dst_idx = dst_y * size + dst_x;
            result[dst_idx] = tile[src_idx];
        }
    }

    result
}
