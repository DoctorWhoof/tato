use tato::video::{TILE_SIZE, TileFlags, TileID};

use crate::*;
use super::*;
use std::collections::HashMap;
use std::{vec, vec::Vec};


// TODO: Move to main engine?
#[derive(Debug, Clone, Copy)]
pub struct TilesetID(pub u8);

pub struct TilesetBuilder {
    pub name: String,
    pub pixels: Vec<u8>,
    // pub palette_hash: HashMap<Color9Bit, u8>,
    // pub palette_id: PaletteID,
    pub tile_hash: HashMap<Vec<u8>, Tile>,
    pub next_tile: u8,
    pub tile_count: u8,
    pub anims: Vec<AnimBuilder>,
    // pub fonts: Vec<Font>,
    // pub tilemaps: Vec<Tilemap>,
    // pub anim_names: Vec<String>,
    // pub font_names: Vec<String>,
    // pub tilemap_names: Vec<String>,
}

impl TilesetBuilder {
    pub fn new(name:String, palette_id: PaletteID) -> Self {
        Self {
            name,
            pixels: vec![],
            // palette_hash: HashMap::new(),
            // palette_id,
            tile_hash: HashMap::new(),
            next_tile: 0,
            tile_count: 0,
            anims: vec![],
            // anim_names: vec![],
            // fonts: vec![],
            // font_names: vec![],
            // tilemaps: vec![],
            // tilemap_names: vec![],
        }
    }

    /// Main workhorse function! Splits images into 8x8 tiles (8 pixels wide, as many pixels as it needs tall).
    /// Returns the indices, the flags and the number of tiles
    pub fn add_tiles(
        &mut self,
        img: &PalettizedImg,
        // group: u8,
        // is_collider: bool,
    ) -> Vec<Tile> {
        let mut tiles = vec![];
        let tile_length = TILE_SIZE as usize * TILE_SIZE as usize;

        let frames_h = img.width / TILE_SIZE as usize;
        let frames_v = img.height / TILE_SIZE as usize;

        // for frame_v in 0..frames_v as usize {
        //     for frame_h in 0..frames_h as usize {
        //         for row in 0..img.rows_per_frame as usize {
        //             for col in 0..img.cols_per_frame as usize {
                for row in 0..frames_v {
                    for col in 0..frames_h {
                        let mut tile_candidate = vec![0u8; tile_length];
                        let mut tile_candidate_flip_h = vec![0u8; tile_length];
                        // let abs_col = (frame_h * img.cols_per_frame as usize) + col;
                        // let abs_row = (frame_v * img.rows_per_frame as usize) + row;

                        for y in 0..TILE_SIZE as usize {
                            for x in 0..TILE_SIZE as usize {
                                let mirror_x = TILE_SIZE as usize - x - 1;
                                let abs_x = (TILE_SIZE as usize * col) + x;
                                let abs_y = (TILE_SIZE as usize * row) + y;
                                let index = (img.width * abs_y) + abs_x;
                                let value = img.pixels[index];
                                tile_candidate[(TILE_SIZE as usize * y) + x] = value;
                                tile_candidate_flip_h[(TILE_SIZE as usize * y) + mirror_x] = value;
                            }
                        }

                        // If hashmap doesn't contain tile, add it
                        if !self.tile_hash.contains_key(&tile_candidate) {
                            // Insert normal tile in hashmap
                            let new_tile = Tile {
                                index: TileID(self.next_tile),
                                flags: TileFlags::default(),
                                // group,
                            };
                            // new_tile.set_collider(is_collider);
                            self.tile_hash
                                .insert(tile_candidate.clone(), new_tile.clone());

                            // Insert horizontally mirrored tile in hashmap
                            let mut tile_flipped_h = new_tile.clone();
                            tile_flipped_h.flags.set_flip_x(true);
                            self.tile_hash.insert(tile_candidate_flip_h, tile_flipped_h);

                            // Add tile pixels to tileset data
                            self.pixels.extend_from_slice(&tile_candidate);

                            // Populates index and attribute vectors that will be returned
                            println!("Creating tile {}", self.next_tile);
                            tiles.push(new_tile);

                            // Next
                            if self.next_tile == 255 {
                                panic!("Error: Tileset capacity exceeded")
                            };
                            self.next_tile += 1;
                            self.tile_count += 1;
                        } else {
                            // If tile is already in hashmap, reuse its index
                            let reused_tile = self.tile_hash.get(&tile_candidate).unwrap();
                            tiles.push((*reused_tile).clone());
                        }
                    }
                }
        //     }
        // }

        tiles
    }
}
