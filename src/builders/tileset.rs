use core::array::from_fn;
use tato_video::*;

use super::*;
use crate::*;
use std::collections::{HashMap, HashSet};
use std::{vec, vec::Vec};

// TODO: Move to main engine?
#[derive(Debug, Clone, Copy)]
pub struct TilesetID(pub u8);

pub struct TilesetBuilder {
    pub allow_tile_transforms: bool,
    pub name: String,
    pub pixels: Vec<u8>,
    // Stores the Tile (ID and Flags) for each unique set of pixels
    pub tile_hash: HashMap<Vec<u8>, TileEntry>,
    // Stores the names of each unique palette
    pub sub_palette_name_hash: HashMap<[u8; SUB_PALETTE_COLOR_COUNT], String>,
    // The actual color indices for the sub-palettes
    pub sub_palettes: Vec<[u8; SUB_PALETTE_COLOR_COUNT]>,
    pub anims: Vec<AnimBuilder>,
    pub single_tiles: Vec<SingleTileBuilder>,
    pub palette_id: PaletteID,
    next_tile: u16,
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
            single_tiles: vec![],
            palette_id,
            sub_palettes: Vec::new(),
            sub_palette_head: 0,
            color_set_to_palette: HashMap::new(),
        }
    }

    fn push_sub_palette(&mut self, sub_palette: &[u8]) -> u8 {
        if self.sub_palette_head == SUB_PALETTE_COUNT {
            panic!(
                "Tileset error: capacity of {} sub-palettes exceeded.",
                self.sub_palettes.len()
            )
        }

        self.sub_palettes
            .push(from_fn(|i| match sub_palette.get(i) {
                Some(value) => *value,
                None => 0,
            }));

        let result = u8::try_from(self.sub_palette_head).unwrap();
        self.sub_palette_head += 1;
        result
    }

    /// Main workhorse function! Splits images into 8x8 tiles (8 pixels wide, as many pixels as it needs tall)
    /// grouped by frame. Returns the indices, the flags and the number of tiles
    pub fn add_tiles(&mut self, img: &PalettizedImg, palette: &PaletteBuilder) -> Vec<TileEntry> {
        // Phase 1: Collect all unique color sets without creating sub-palettes yet
        let color_sets = self.collect_tile_color_sets(img);

        // Phase 2: Allocate optimal sub-palettes
        self.allocate_sub_palettes(&color_sets, palette);

        // Phase 3: Process tiles with pre-allocated sub-palettes
        self.process_tiles_with_palettes(img)
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

                                if value != 0 {
                                    // Typically 0 is transparent/background
                                    colors.insert(value);
                                }
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
        self.sub_palette_name_hash.clear();
        self.sub_palettes.clear();
        self.sub_palette_head = 0;

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
                    if color != 0 {
                        combined_palette.insert(color);
                    }
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
    fn process_tiles_with_palettes(&mut self, img: &PalettizedImg) -> Vec<TileEntry> {
        let mut tiles = vec![];
        let tile_length = TILE_SIZE as usize * TILE_SIZE as usize;

        for frame_v in 0..img.frames_v as usize {
            for frame_h in 0..img.frames_h as usize {
                for row in 0..img.rows_per_frame as usize {
                    for col in 0..img.cols_per_frame as usize {
                        let abs_col = (frame_h * img.cols_per_frame as usize) + col;
                        let abs_row = (frame_v * img.rows_per_frame as usize) + row;

                        let mut tile_candidate = vec![0u8; tile_length];
                        let mut tile_candidate_flip_h = vec![0u8; tile_length];
                        let mut color_set = HashSet::new();

                        // Extract tile pixels and collect colors
                        for y in 0..TILE_SIZE as usize {
                            for x in 0..TILE_SIZE as usize {
                                let mirror_x = TILE_SIZE as usize - x - 1;
                                let abs_x = (TILE_SIZE as usize * abs_col) + x;
                                let abs_y = (TILE_SIZE as usize * abs_row) + y;
                                let index = (img.width * abs_y) + abs_x;
                                let value = img.pixels[index];

                                tile_candidate[(TILE_SIZE as usize * y) + x] = value;
                                tile_candidate_flip_h[(TILE_SIZE as usize * y) + mirror_x] = value;

                                if value != 0 {
                                    color_set.insert(value);
                                }
                            }
                        }

                        // Rest of processing remains similar to the original
                        // Look up or create new tile

                        if self.tile_hash.contains_key(&tile_candidate) {
                            // If tile is already in hashmap, reuse its index
                            let reused_tile = self.tile_hash.get(&tile_candidate).unwrap();
                            tiles.push((*reused_tile).clone());
                        } else {
                            // If hashmap doesn't contain tile, add it
                            if self.next_tile == 256 {
                                panic!("Error: Tileset capacity exceeded")
                            };

                            // Insert normal tile in hashmap
                            let new_tile = TileEntry {
                                id: TileID(self.next_tile),
                                flags: TileFlags::default(),
                            };

                            self.tile_hash
                                .insert(tile_candidate.clone(), new_tile.clone());

                            // Insert horizontally mirrored tile in hashmap if allowed
                            if self.allow_tile_transforms {
                                let mut tile_flipped_h = new_tile.clone();
                                tile_flipped_h.flags.set_flip_x(true);
                                self.tile_hash.insert(tile_candidate_flip_h, tile_flipped_h);
                            }

                            // Add tile pixels to tileset data
                            self.pixels.extend_from_slice(&tile_candidate);

                            // Add to return vector
                            tiles.push(new_tile);

                            // Next
                            self.next_tile += 1;
                        }
                    }
                }
            }
        }

        tiles
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
    colors
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("_")
}
