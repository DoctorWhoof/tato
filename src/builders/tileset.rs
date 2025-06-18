use core::array::from_fn;
use tato_video::*;

use super::*;
use crate::*;
use std::collections::{HashMap, HashSet};

const TILE_LEN: usize = TILE_SIZE as usize * TILE_SIZE as usize;
type TileData = [u8; TILE_LEN];

#[derive(Debug, Clone, Copy)]
pub struct TilesetBuilderID(pub u8);

pub struct TilesetBuilder {
    pub allow_tile_transforms: bool,
    pub name: String,
    pub pixels: Vec<u8>,
    pub tile_hash: HashMap<TileData, Cell>,
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
        // Iterate frames, then tiles within frames.
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

                        // Get or create tile
                        let mut colors = HashSet::new();
                        for &pixel in &tile_data {
                            colors.insert(pixel);
                        }

                        if colors.len() > SUB_PALETTE_COLOR_COUNT {
                            panic!("Tile exceeds {} color limit", SUB_PALETTE_COLOR_COUNT);
                        }

                        // Find or create sub-palette
                        let mut sub_palette_id = None;
                        for (i, sub_pal) in self.sub_palettes.iter().enumerate() {
                            let pal_colors: HashSet<u8> = sub_pal.iter().cloned().collect();
                            if colors.is_subset(&pal_colors) {
                                sub_palette_id = Some(i as u8);
                                break;
                            }
                        }

                        let sub_palette_id = match sub_palette_id {
                            Some(id) => id,
                            None => {
                                // Create new sub-palette
                                if self.sub_palette_head >= SUB_PALETTE_COUNT {
                                    panic!("Sub-palette capacity {} exceeded", SUB_PALETTE_COUNT);
                                }

                                let mut color_vec: Vec<_> = colors.iter().cloned().collect();
                                color_vec.sort();

                                let palette_array =
                                    from_fn(|i| if i < color_vec.len() { color_vec[i] } else { 0 });

                                self.sub_palettes.push(palette_array);
                                let palette_id = self.sub_palette_head as u8;
                                self.sub_palette_head += 1;

                                // Set name
                                let name = format!("{}_{}", palette.name, palette_id);
                                self.sub_palette_name_hash.insert(palette_array, name);

                                palette_id
                            },
                        };

                        let sub_palette = &self.sub_palettes[sub_palette_id as usize];

                        // Normalize to sub-palette indices
                        let mut normalized = [0u8; TILE_LEN];
                        for (i, &original_color) in tile_data.iter().enumerate() {
                            normalized[i] = sub_palette
                                .iter()
                                .position(|&pal_color| pal_color == original_color)
                                .unwrap_or(0) as u8;
                        }

                        // Check if this tile (or any transformation) exists
                        let mut found_tile = None;

                        // Check original first
                        if let Some(existing) = self.tile_hash.get(&normalized) {
                            found_tile = Some((*existing, existing.flags));
                        } else if self.allow_tile_transforms {
                            // Try all 7 other transformations
                            'outer: for flip_x in [false, true] {
                                for flip_y in [false, true] {
                                    for rotation in [false, true] {
                                        if !flip_x && !flip_y && !rotation {
                                            continue;
                                        }

                                        let transformed =
                                            transform_tile(&normalized, flip_x, flip_y, rotation);
                                        if let Some(existing) = self.tile_hash.get(&transformed) {
                                            found_tile = Some((*existing, existing.flags));
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                        }

                        let cell = match found_tile {
                            Some((existing_tile, flags)) => {
                                let mut cell = Cell { id: existing_tile.id, flags };
                                cell.flags.set_palette(PaletteID(sub_palette_id));
                                cell
                            },
                            None => {
                                // Create new tile
                                let mut new_tile = Cell {
                                    id: TileID(self.next_tile),
                                    flags: TileFlags::default(),
                                };
                                new_tile.flags.set_palette(PaletteID(sub_palette_id));

                                // Store original
                                self.tile_hash.insert(normalized, new_tile);

                                // Store all transformations
                                if self.allow_tile_transforms {
                                    for flip_x in [false, true] {
                                        for flip_y in [false, true] {
                                            for rotation in [false, true] {
                                                if !flip_x && !flip_y && !rotation {
                                                    continue;
                                                }

                                                let transformed = transform_tile(
                                                    &normalized,
                                                    flip_x,
                                                    flip_y,
                                                    rotation,
                                                );
                                                let mut tile_with_flags = Cell {
                                                    id: new_tile.id,
                                                    flags: TileFlags::default(),
                                                };
                                                tile_with_flags.flags.set_flip_x(flip_x);
                                                tile_with_flags.flags.set_flip_y(flip_y);
                                                tile_with_flags.flags.set_rotation(rotation);

                                                self.tile_hash.insert(transformed, tile_with_flags);
                                            }
                                        }
                                    }
                                }

                                // Add to pixel data
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
