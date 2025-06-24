use core::array::from_fn;
use tato_video::*;

use super::*;
use crate::*;
use std::collections::{HashMap, HashSet};

const TILE_LEN: usize = TILE_SIZE as usize * TILE_SIZE as usize;
type TileData = [u8; TILE_LEN];
type CanonicalTile = [u8; TILE_LEN]; // Colors remapped to canonical form (0,1,2,3...) to allow detection of palette-swapped tiles!

#[derive(Debug, Clone, Copy)]
pub struct TilesetBuilderID(pub u8);

pub struct TilesetBuilder {
    pub allow_tile_transforms: bool,
    pub name: String,
    pub pixels: Vec<u8>,
    pub tile_hash: HashMap<CanonicalTile, Cell>,
    pub sub_palette_name_hash: HashMap<[u8; SUB_PALETTE_COLOR_COUNT], String>,
    pub sub_palettes: Vec<[u8; SUB_PALETTE_COLOR_COUNT]>,
    pub anims: Vec<AnimBuilder>,
    pub maps: Vec<MapBuilder>,
    pub single_tiles: Vec<SingleTileBuilder>,
    pub palette_id: PaletteID,
    next_tile: u8,
    sub_palette_head: usize,
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
        }
    }

    pub fn add_tiles(&mut self, img: &PalettizedImg, palette: &PaletteBuilder) -> Vec<Cell> {
        let mut tiles = vec![];

        // Main detection routine.
        // Iterate animation frames, then tiles within frames.
        for frame_v in 0..img.frames_v as usize {
            for frame_h in 0..img.frames_h as usize {
                for row in 0..img.rows_per_frame as usize {
                    for col in 0..img.cols_per_frame as usize {
                        // Extract tile pixels
                        let mut tile_data = [0u8; TILE_LEN];
                        let abs_col = (frame_h * img.cols_per_frame as usize) + col;
                        let abs_row = (frame_v * img.rows_per_frame as usize) + row;

                        for y in 0..TILE_SIZE as usize {
                            for x in 0..TILE_SIZE as usize {
                                let abs_x = (TILE_SIZE as usize * abs_col) + x;
                                let abs_y = (TILE_SIZE as usize * abs_row) + y;
                                let index = (img.width * abs_y) + abs_x;
                                tile_data[(TILE_SIZE as usize * y) + x] = img.pixels[index];
                            }
                        }

                        // Create canonical representation
                        let (canonical_tile, color_mapping) = create_canonical_tile(&tile_data);

                        if color_mapping.len() > SUB_PALETTE_COLOR_COUNT {
                            panic!("Tile exceeds {} color limit", SUB_PALETTE_COLOR_COUNT);
                        }

                        // Will check if this canonical tile (or any transformation) exists
                        let mut found_cell = None;

                        // Create a temporary remapping to check against hash
                        let (temp_sub_palette_id, temp_remapping) = self.find_or_create_compatible_sub_palette(&color_mapping, palette);
                        let mut temp_normalized = [0u8; TILE_LEN];
                        for (i, &canonical_index) in canonical_tile.iter().enumerate() {
                            temp_normalized[i] = temp_remapping[canonical_index as usize];
                        }

                        // Check original first using remapped data
                        if let Some(existing) = self.tile_hash.get(&temp_normalized) {
                            found_cell = Some(*existing);
                        } else if self.allow_tile_transforms {
                            // Try all 7 other transformations using remapped data
                            'outer: for flip_x in [false, true] {
                                for flip_y in [false, true] {
                                    for rotation in [false, true] {
                                        if !flip_x && !flip_y && !rotation {
                                            continue;
                                        }

                                        let transformed_original = transform_tile(
                                            &tile_data,
                                            flip_x,
                                            flip_y,
                                            rotation,
                                        );
                                        let (transformed_canonical, transformed_colors) = create_canonical_tile(&transformed_original);

                                        // Apply remapping to transformed data
                                        let mut transformed_normalized = [0u8; TILE_LEN];
                                        for (i, &canonical_index) in transformed_canonical.iter().enumerate() {
                                            if (canonical_index as usize) < transformed_colors.len() {
                                                let color = transformed_colors[canonical_index as usize];
                                                let original_index = color_mapping.iter().position(|&c| c == color).unwrap_or(0);
                                                transformed_normalized[i] = temp_remapping[original_index];
                                            } else {
                                                transformed_normalized[i] = 0;
                                            }
                                        }

                                        if let Some(existing) = self.tile_hash.get(&transformed_normalized) {
                                            found_cell = Some(*existing);
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                        }

                        let cell = match found_cell {
                            Some(existing_cell) => {
                                // Found existing tile with same pattern
                                // Use the same sub-palette mapping we used for lookup
                                let mut cell = Cell {
                                    id: existing_cell.id,
                                    flags: existing_cell.flags,
                                };
                                cell.flags.set_palette(PaletteID(temp_sub_palette_id));
                                cell
                            },
                            None => {
                                // Create new tile - but still try to reuse compatible sub-palette
                                let (sub_palette_id, remapping) = self.find_or_create_compatible_sub_palette(&color_mapping, palette);

                                let mut new_tile = Cell {
                                    id: TileID(self.next_tile),
                                    flags: TileFlags::default(),
                                };
                                new_tile.flags.set_palette(PaletteID(sub_palette_id));

                                // Store canonical tile data remapped to sub-palette indices
                                let mut normalized = [0u8; TILE_LEN];
                                for (i, &canonical_index) in canonical_tile.iter().enumerate() {
                                    normalized[i] = remapping[canonical_index as usize];
                                }
                                self.pixels.extend_from_slice(&normalized);

                                // Store remapped tile in hash (after remapping is complete)
                                self.tile_hash.insert(normalized, new_tile);

                                // Store all transformations using remapped data
                                if self.allow_tile_transforms {
                                    for flip_x in [false, true] {
                                        for flip_y in [false, true] {
                                            for rotation in [false, true] {
                                                if !flip_x && !flip_y && !rotation {
                                                    continue;
                                                }

                                                let transformed_original = transform_tile(
                                                    &tile_data,
                                                    flip_x,
                                                    flip_y,
                                                    rotation,
                                                );
                                                let (transformed_canonical, transformed_colors) = create_canonical_tile(&transformed_original);

                                                // Apply same remapping to transformed data
                                                let mut transformed_normalized = [0u8; TILE_LEN];
                                                for (i, &canonical_index) in transformed_canonical.iter().enumerate() {
                                                    if (canonical_index as usize) < transformed_colors.len() {
                                                        // Find this color in our original color mapping
                                                        let color = transformed_colors[canonical_index as usize];
                                                        let original_index = color_mapping.iter().position(|&c| c == color).unwrap_or(0);
                                                        transformed_normalized[i] = remapping[original_index];
                                                    } else {
                                                        transformed_normalized[i] = 0;
                                                    }
                                                }

                                                // Only store if this transformation produces different data
                                                if !self.tile_hash.contains_key(&transformed_normalized) {
                                                    let mut cell_with_flags = new_tile;
                                                    cell_with_flags.flags.set_flip_x(flip_x);
                                                    cell_with_flags.flags.set_flip_y(flip_y);
                                                    cell_with_flags.flags.set_rotation(rotation);

                                                    self.tile_hash.insert(transformed_normalized, cell_with_flags);
                                                }
                                            }
                                        }
                                    }
                                }
                                self.next_tile += 1;

                                new_tile
                            },
                        };

                        tiles.push(cell);
                    }
                }
            }
        }

        tiles
    }

    fn find_or_create_compatible_sub_palette(&mut self, colors: &[u8], palette: &PaletteBuilder) -> (u8, Vec<u8>) {
        // Work with unique colors only to avoid issues with repeated colors
        let unique_colors: Vec<u8> = {
            let mut seen = HashSet::new();
            colors.iter().filter(|&&color| seen.insert(color)).cloned().collect()
        };

        // Check for exact match first (cheapest check)
        let target_palette_array = from_fn(|i| if i < unique_colors.len() { unique_colors[i] } else { 0 });
        for (i, sub_pal) in self.sub_palettes.iter().enumerate() {
            if *sub_pal == target_palette_array {
                // Create identity remapping for our original colors (including duplicates)
                let mut remapping = Vec::new();
                for &color in colors {
                    let unique_index = unique_colors.iter().position(|&c| c == color).unwrap_or(0);
                    remapping.push(unique_index as u8);
                }
                return (i as u8, remapping);
            }
        }

        // Try to find an existing sub-palette that contains all our unique colors
        let color_set: HashSet<u8> = unique_colors.iter().cloned().collect();
        for (i, sub_pal) in self.sub_palettes.iter().enumerate() {
            let pal_colors: HashSet<u8> = sub_pal.iter().filter(|&&c| c != 0 || sub_pal[0] == 0).cloned().collect();
            if color_set.is_subset(&pal_colors) {
                // Create remapping from our canonical indices to sub-palette indices
                let mut remapping = Vec::new();
                for &color in colors {
                    let sub_pal_index = sub_pal.iter().position(|&pal_color| pal_color == color).unwrap_or(0);
                    remapping.push(sub_pal_index as u8);
                }
                return (i as u8, remapping);
            }
        }

        // Create new sub-palette with unique colors only
        if self.sub_palette_head >= SUB_PALETTE_COUNT {
            panic!("Sub-palette capacity {} exceeded", SUB_PALETTE_COUNT);
        }

        self.sub_palettes.push(target_palette_array);
        let palette_id = self.sub_palette_head as u8;
        self.sub_palette_head += 1;

        // Set name
        let name = format!("{}_{}", palette.name, palette_id);
        self.sub_palette_name_hash.insert(target_palette_array, name);

        // Create identity remapping for our original colors (including duplicates)
        let mut remapping = Vec::new();
        for &color in colors {
            let unique_index = unique_colors.iter().position(|&c| c == color).unwrap_or(0);
            remapping.push(unique_index as u8);
        }
        (palette_id, remapping)
    }
}

fn create_canonical_tile(tile_data: &TileData) -> (CanonicalTile, Vec<u8>) {
    let mut canonical = [0u8; TILE_LEN];
    let mut color_mapping = Vec::new();
    let mut color_to_index = HashMap::new();

    for (i, &color) in tile_data.iter().enumerate() {
        let canonical_index = if let Some(&index) = color_to_index.get(&color) {
            index
        } else {
            let new_index = color_mapping.len() as u8;
            color_mapping.push(color);
            color_to_index.insert(color, new_index);
            new_index
        };
        canonical[i] = canonical_index;
    }

    (canonical, color_mapping)
}

fn transform_tile(tile: &TileData, flip_x: bool, flip_y: bool, rotation: bool) -> TileData {
    let mut result = [0u8; TILE_LEN];
    let size = TILE_SIZE as usize;

    for y in 0..size {
        for x in 0..size {
            let src_idx = y * size + x;
            let mut dst_x = x;
            let mut dst_y = y;

            if rotation {
                let temp = dst_x;
                dst_x = dst_y;
                dst_y = size - 1 - temp;
            }
            if flip_x {
                dst_x = size - 1 - dst_x;
            }
            if flip_y {
                dst_y = size - 1 - dst_y;
            }

            result[dst_y * size + dst_x] = tile[src_idx];
        }
    }
    result
}
