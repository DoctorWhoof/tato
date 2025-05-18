use crate::{Tileset, TilesetID};
use core::array::from_fn;
use tato_video::*;

const TILE_BANK_LEN: u16 = 512;

/// Stores multiple tilesets. The const generic BANKS determines how many tilesets
/// it contains, and "costs" about 8Kb per tileset (max 512 tiles per tileset).
#[derive(Debug)]
pub struct TileBank<const BANKS: usize> {
    // An array of arrays of tiles
    pub tiles: [[Tile<2>; TILE_BANK_LEN as usize]; BANKS], // The tiles (pixels) for each bank
    // The tile head for each bank
    tile_head: [u16; BANKS],
    // Max 256 regardless of which bank it's stored. TODO: May bump to u16!
    pub sets: [Tileset; 256],
    // Tileset counter
    set_head: u8,
}

impl<const BANKS: usize> Default for TileBank<BANKS> {
    fn default() -> Self {
        assert!(BANKS <= u8::MAX as usize, err!("Tile Banks capacity of {} exceeded"), u8::MAX);
        Self {
            tiles: from_fn(|_| from_fn(|_| Tile::<2>::default())),
            sets: from_fn(|_| Tileset::default()),
            set_head: 0,
            tile_head: from_fn(|_| 0),
        }
    }
}

impl<const BANKS: usize> TileBank<BANKS> {
    pub fn reset(&mut self) {
        for head in &mut self.tile_head {
            *head = 0;
        }
        self.set_head = 0
    }

    pub fn new_tile(&mut self, bank: u8, tile: &Tile<2>) -> TileID {
        assert!((bank as usize) < BANKS, err!("Invalid bank index {}"), bank);
        assert!((self.set_head as usize) < BANKS, err!("Tileset capacity reached"));
        let tile_head = self.tile_head[bank as usize];
        let result = TileID(tile_head);
        // Copy tile data to bank
        let dest_index = tile_head as usize;
        self.tiles[bank as usize][dest_index] = tile.clone();
        self.tile_head[bank as usize] += 1;
        result
    }

    pub fn new_tileset(&mut self, bank: u8, data: &[Tile<2>]) -> TilesetID {
        assert!((bank as usize) < BANKS, err!("Invalid bank index {}"), bank);
        assert!((self.set_head as usize) < BANKS, err!("Tileset capacity reached"));
        let tiles_start = self.tile_head[bank as usize];
        let result = TilesetID(self.set_head);
        let set_len = u16::try_from(data.len()).unwrap();
        assert!(
            tiles_start as usize + set_len as usize <= TILE_BANK_LEN as usize,
            err!("Bank {} tile capacity exceeded, tried to insert {} but only had {} slots"),
            bank,
            set_len,
            TILE_BANK_LEN - tiles_start
        );
        // Copy tile data to bank
        for (i, tile) in data.iter().enumerate() {
            let dest_index = tiles_start as usize + i;
            self.tiles[bank as usize][dest_index] = tile.clone();
        }
        // Create tileset entry
        self.sets[self.set_head as usize] = Tileset { start: tiles_start, len: set_len, bank };
        // Advance and return
        self.set_head += 1;
        self.tile_head[bank as usize] += set_len;
        result
    }

    pub fn get_all(&self) -> [&[Tile<2>]; BANKS] {
        core::array::from_fn(|i| &self.tiles[i][..])
    }

    // pub fn get_tileset_mut(&mut self, id: TilesetID) -> Option<&mut Tileset<TILESET_LEN>> {
    //     if id.0 < self.head {
    //         let set = &mut self.data[id.0 as usize];
    //         return Some(set);
    //     }
    //     None
    // }

    // pub fn get(&self, id: TilesetID) -> Option<&[Tile<2>]> {
    //     if id.0 < self.head {
    //         let set = &self.data[id.0 as usize];
    //         let start = 0; // TODO: Subsets that can start not on zero
    //         let end = set.len();
    //         return Some(&set.tiles[start..end]);
    //     }
    //     None
    // }

    // pub fn new_tile(&mut self, tileset: TilesetID, data: &[u8]) -> TileID {
    //     // Check if number of pixels is correct
    //     assert!(
    //         data.len() == TILE_PIXEL_COUNT,
    //         err!("Tile data length must match TILE_PIXEL_COUNT ({})"),
    //         TILE_PIXEL_COUNT
    //     );

    //     // Check if we have enough space
    //     if (self.head as usize) >= BANKS {
    //         panic!(err!("Not enough space for new tile"))
    //     }

    //     let tile_id = self.head;

    //     // Pack 8 pixels (2 bits each) into each cluster
    //     // TODO: REPLACE WITH TILE.SET_PIXEL
    //     let mut cluster_index = 0;
    //     let mut subpixel_index = 0;
    //     for i in 0..TILE_PIXEL_COUNT {
    //         // Clamp color to maximum allowed
    //         let value = data[i].clamp(0, COLORS_PER_TILE as u8);

    //         // Set pixel data
    //         self.tiles[self.head as usize] //
    //             .clusters[cluster_index]
    //             .set_subpixel(value, subpixel_index);

    //         // Advance
    //         subpixel_index += 1;
    //         if subpixel_index >= PIXELS_PER_CLUSTER {
    //             subpixel_index = 0;
    //             cluster_index += 1;
    //         }
    //     }

    //     self.head += 1;
    //     TileID(tile_id)
    // }
}
