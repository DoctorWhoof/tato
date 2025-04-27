use core::array::from_fn;
use tato::video::{TILE_SIZE, TileFlags, TileID};

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
    pub tile_hash: HashMap<Vec<u8>, Tile>,
    // Stores the names of each unique palette
    pub sub_palette_name_hash: HashMap<[u8; SUB_PALETTE_COLOR_COUNT], String>,
    // The actual color indices for the sub-palettes
    pub sub_palettes: Vec<[u8; SUB_PALETTE_COLOR_COUNT]>,
    pub anims: Vec<AnimBuilder>,
    pub palette_id: PaletteID,
    next_tile: u16,
    sub_palette_head: usize,
}

impl TilesetBuilder {
    pub fn new(name: String, palette_id:PaletteID) -> Self {
        Self {
            allow_tile_transforms: true,
            name,
            pixels: vec![],
            tile_hash: HashMap::new(),
            sub_palette_name_hash: HashMap::new(),
            next_tile: 0,
            anims: vec![],
            palette_id,
            sub_palettes: Vec::new(),
            sub_palette_head: 0,
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
    pub fn add_tiles(
        &mut self,
        img: &PalettizedImg,
        palette: &PaletteBuilder,
        // group: u8,
        // is_collider: bool,
    ) -> Vec<Tile> {
        let mut tiles = vec![];
        let tile_length = TILE_SIZE as usize * TILE_SIZE as usize;

        for frame_v in 0..img.frames_v as usize {
            for frame_h in 0..img.frames_h as usize {
                for row in 0..img.rows_per_frame as usize {
                    for col in 0..img.cols_per_frame as usize {
                        let abs_col = (frame_h * img.cols_per_frame as usize) + col;
                        let abs_row = (frame_v * img.rows_per_frame as usize) + row;
                        // println!("Processing {},{}", abs_col, abs_row);

                        let mut tile_candidate = vec![0u8; tile_length];
                        let mut tile_candidate_flip_h = vec![0u8; tile_length];
                        let mut sub_palette_candidate = Vec::<u8>::new();
                        let mut sub_palette_hash = HashSet::<u8>::new();
                        for y in 0..TILE_SIZE as usize {
                            for x in 0..TILE_SIZE as usize {
                                let mirror_x = TILE_SIZE as usize - x - 1;
                                let abs_x = (TILE_SIZE as usize * abs_col) + x;
                                let abs_y = (TILE_SIZE as usize * abs_row) + y;
                                let index = (img.width * abs_y) + abs_x;
                                let value = img.pixels[index];
                                tile_candidate[(TILE_SIZE as usize * y) + x] = value;
                                tile_candidate_flip_h[(TILE_SIZE as usize * y) + mirror_x] = value;

                                // Subpalette handling
                                if !sub_palette_hash.contains(&value) {
                                    if sub_palette_hash.len() >= SUB_PALETTE_COLOR_COUNT {
                                        panic!(
                                            "Tile at position ({}, {}) exceeds the {} color limit",
                                            abs_col, abs_row, SUB_PALETTE_COLOR_COUNT
                                        );
                                    }

                                    sub_palette_hash.insert(value);
                                    sub_palette_candidate.push(value);
                                }
                            }
                        }


                        // Sort and Convert Vec to array
                        sub_palette_candidate.sort();
                        let offset = SUB_PALETTE_COLOR_COUNT - sub_palette_candidate.len();
                        let sub_palette_candidate: [u8; SUB_PALETTE_COLOR_COUNT] = from_fn(|i| {
                            if i >= offset {
                                match sub_palette_candidate.get(i + offset) {
                                    Some(value) => *value,
                                    None => 0,
                                }
                            } else {
                                0
                            }
                        });

                        if !self
                            .sub_palette_name_hash
                            .contains_key(&sub_palette_candidate)
                        {
                            let sum: u8 = sub_palette_candidate.iter().sum();
                            // Prevents adding empty sub palettes
                            if sum > 0 {
                                let index = self.push_sub_palette(sub_palette_candidate.as_slice());
                                let name = format!("{}_{}", palette.name, index);
                                println!("Name: {}", name);
                                self.sub_palette_name_hash
                                    .insert(sub_palette_candidate, name);
                            }
                        }

                        // println!("{:?}", tile_candidate);
                        if self.tile_hash.contains_key(&tile_candidate) {
                            // If tile is already in hashmap, reuse its index
                            let reused_tile = self.tile_hash.get(&tile_candidate).unwrap();
                            println!(
                                "\tReusing tile {} from source tiles {},{}",
                                reused_tile.index.0, abs_col, abs_row
                            );
                            tiles.push((*reused_tile).clone());
                        } else {
                            // If hashmap doesn't contain tile, add it
                            if self.next_tile == 256 {
                                panic!("Error: Tileset capacity exceeded")
                            };

                            // Insert normal tile in hashmap
                            let new_tile = Tile {
                                index: TileID(self.next_tile as u8),
                                flags: TileFlags::default(),
                                // group,
                            };
                            // new_tile.set_collider(is_collider);
                            self.tile_hash
                                .insert(tile_candidate.clone(), new_tile.clone());

                            println!(
                                "\tNew tile {} from source tiles {},{}",
                                self.next_tile, abs_col, abs_row
                            );
                            // Insert horizontally mirrored tile in hashmap
                            if self.allow_tile_transforms{
                                let mut tile_flipped_h = new_tile.clone();
                                tile_flipped_h.flags.set_flip_x(true);
                                self.tile_hash.insert(tile_candidate_flip_h, tile_flipped_h);
                            }

                            // Add tile pixels to tileset data
                            self.pixels.extend_from_slice(&tile_candidate);

                            // Populates index and attribute vectors that will be returned
                            // println!("Creating tile {}", self.next_tile);
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
