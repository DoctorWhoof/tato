use core::array::from_fn;
use tato_video::*;

use super::*;
use crate::*;
use std::collections::{HashMap, HashSet};

const TILE_LEN: usize = TILE_SIZE as usize * TILE_SIZE as usize;
type TileData = [u8; TILE_LEN];
// Colors remapped to canonical form (0,1,2,3...) to allow detection of palette-swapped tiles!
type CanonicalTile = [u8; TILE_LEN];

#[derive(Debug, Clone, Copy)]
pub struct TilesetBuilderID(pub u8);

#[derive(Debug, Clone)]
pub struct TileInfo {
    cell: Cell,
    original_colors: Vec<u8>, // The original colors in canonical order
}

pub struct TilesetBuilder {
    pub allow_tile_transforms: bool,
    pub name: String,
    pub pixels: Vec<u8>,
    pub tile_hash: HashMap<CanonicalTile, TileInfo>,
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

                        // Get unique colors and check limits
                        let mut colors = HashSet::new();
                        for &pixel in &tile_data {
                            colors.insert(pixel);
                        }

                        if colors.len() > SUB_PALETTE_COLOR_COUNT {
                            panic!("Tile exceeds {} color limit", SUB_PALETTE_COLOR_COUNT);
                        }

                        // Create canonical representation
                        let (canonical_tile, color_mapping) = create_canonical_tile(&tile_data);

                        // Check if this canonical tile (or any transformation) exists
                        let mut found_tile_info = None;

                        // Check original first
                        if let Some(existing) = self.tile_hash.get(&canonical_tile) {
                            found_tile_info = Some(existing.clone());
                        } else if self.allow_tile_transforms {
                            // Try all 7 other transformations
                            'outer: for flip_x in [false, true] {
                                for flip_y in [false, true] {
                                    for rotation in [false, true] {
                                        if !flip_x && !flip_y && !rotation {
                                            continue;
                                        }

                                        let transformed = transform_tile(
                                            &canonical_tile,
                                            flip_x,
                                            flip_y,
                                            rotation,
                                        );
                                        if let Some(existing) = self.tile_hash.get(&transformed) {
                                            found_tile_info = Some(existing.clone());
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                        }

                        let cell = match found_tile_info {
                            Some(existing_tile_info) => {
                                // Found existing tile with same pattern
                                // Try to reuse existing sub-palette or find compatible one
                                let sub_palette_id = self.find_or_create_compatible_sub_palette(&color_mapping, palette);

                                let mut cell = Cell {
                                    id: existing_tile_info.cell.id,
                                    flags: existing_tile_info.cell.flags,
                                };
                                cell.flags.set_palette(PaletteID(sub_palette_id));
                                cell
                            },
                            None => {
                                // Create new tile - but still try to reuse compatible sub-palette
                                let sub_palette_id = self.find_or_create_compatible_sub_palette(&color_mapping, palette);

                                let mut new_tile = Cell {
                                    id: TileID(self.next_tile),
                                    flags: TileFlags::default(),
                                };
                                new_tile.flags.set_palette(PaletteID(sub_palette_id));

                                let tile_info = TileInfo {
                                    cell: new_tile,
                                    original_colors: color_mapping.clone(),
                                };

                                // Store original canonical tile
                                self.tile_hash.insert(canonical_tile, tile_info.clone());

                                // Store all transformations
                                if self.allow_tile_transforms {
                                    for flip_x in [false, true] {
                                        for flip_y in [false, true] {
                                            for rotation in [false, true] {
                                                if !flip_x && !flip_y && !rotation {
                                                    continue;
                                                }

                                                let transformed = transform_tile(
                                                    &canonical_tile,
                                                    flip_x,
                                                    flip_y,
                                                    rotation,
                                                );

                                                let mut tile_info_with_flags = tile_info.clone();
                                                tile_info_with_flags.cell.flags.set_flip_x(flip_x);
                                                tile_info_with_flags.cell.flags.set_flip_y(flip_y);
                                                tile_info_with_flags
                                                    .cell
                                                    .flags
                                                    .set_rotation(rotation);

                                                self.tile_hash
                                                    .insert(transformed, tile_info_with_flags);
                                            }
                                        }
                                    }
                                }

                                // Create normalized pixel data for the chosen sub-palette
                                let sub_palette = &self.sub_palettes[sub_palette_id as usize];
                                let mut normalized = [0u8; TILE_LEN];
                                for (i, &original_color) in tile_data.iter().enumerate() {
                                    normalized[i] = sub_palette
                                        .iter()
                                        .position(|&pal_color| pal_color == original_color)
                                        .unwrap_or(0) as u8;
                                }

                                // Add normalized tile to pixel data
                                self.pixels.extend_from_slice(&normalized);
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

    fn find_or_create_compatible_sub_palette(&mut self, colors: &[u8], palette: &PaletteBuilder) -> u8 {
        let color_set: HashSet<u8> = colors.iter().cloned().collect();

        // First, try to find an existing sub-palette that contains all our colors
        for (i, sub_pal) in self.sub_palettes.iter().enumerate() {
            let pal_colors: HashSet<u8> = sub_pal.iter().cloned().collect();
            if color_set.is_subset(&pal_colors) {
                return i as u8;
            }
        }

        // Check for exact match to avoid duplicates
        let target_palette_array = from_fn(|i| if i < colors.len() { colors[i] } else { 0 });
        for (i, sub_pal) in self.sub_palettes.iter().enumerate() {
            if *sub_pal == target_palette_array {
                return i as u8;
            }
        }

        // Create new sub-palette
        if self.sub_palette_head >= SUB_PALETTE_COUNT {
            panic!("Sub-palette capacity {} exceeded", SUB_PALETTE_COUNT);
        }

        self.sub_palettes.push(target_palette_array);
        let palette_id = self.sub_palette_head as u8;
        self.sub_palette_head += 1;

        // Set name
        let name = format!("{}_{}", palette.name, palette_id);
        self.sub_palette_name_hash.insert(target_palette_array, name);

        palette_id
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
