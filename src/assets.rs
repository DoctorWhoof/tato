use crate::*;
use core::array::from_fn;

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct MapID(pub u8);

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct AnimID(pub u8);

#[derive(Debug, Clone, Copy, Default)]
pub struct Tileset {
    pub bank_id: u8,
    pub tile_start: u16,
    pub tiles_count: u16,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Anim {
    pub bank_id: u8,
    pub fps: u8,
    pub columns_per_frame: u8,
    pub rows_per_frame: u8,
    pub data_start: u16,
    pub data_len: u16,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Tilemap {
    pub bank_id: u8,
    pub columns: u16,
    pub rows: u16,
    pub data_start: u16,
    pub data_len: u16,
}

#[derive(Debug, Clone)]
pub struct AssetRoster {
    pub tilesets: [Tileset; 256],
    pub anims: [Anim; 256],
    // Stores any asset that requires cells, like Anims and Maps
    pub cells: [Cell; 2048],
    pub maps: [Tilemap; 256],
    // Everything that needs to be counted.
    cell_head: u16,
    tileset_head: u8,
    anim_head: u8,
    map_head: u8,
}

impl AssetRoster {
    pub fn new() -> Self {
        Self {
            // Metadata
            tilesets: from_fn(|_| Tileset::default()),
            anims: from_fn(|_| Anim::default()),
            maps: core::array::from_fn(|_| Tilemap::default()),
            // "Flat" entry data for maps and anims
            cells: from_fn(|_| Cell::default()),
            // frame_head: 0,
            cell_head: 0,
            tileset_head: 0,
            anim_head: 0,
            map_head: 0,
        }
    }

    pub fn reset(&mut self) {
        self.cell_head = 0;
        self.tileset_head = 0;
        self.anim_head = 0;
        self.map_head = 0;
    }
}

impl Tato {
    /// Adds a single tile, returns a TileID
    #[inline]
    pub fn add_tile(&mut self, bank_id: u8, tile: &Tile<2>) -> TileID {
        self.banks[bank_id as usize].add_tile(tile)
    }

    /// Adds a tileset as a batch of tiles to the bank
    /// Returns the tileset id.
    pub fn add_tileset(&mut self, bank_id: u8, tiles: &[Tile<2>]) -> Option<TilesetID> {
        let bank = self.banks.get_mut(bank_id as usize)?;
        let roster = &mut self.assets;
        if bank.tile_count() + tiles.len() > bank.tile_capacity() {
            return None;
        }

        let id = roster.tileset_head;
        let tile_start = u16::try_from(bank.tile_count()).unwrap();
        let tiles_count = u16::try_from(tiles.len()).unwrap();

        for tile in tiles.iter() {
            bank.add_tile(tile);
        }

        roster.tilesets[id as usize] = Tileset { bank_id, tile_start, tiles_count };
        roster.tileset_head += 1;
        Some(TilesetID(id))
    }


    /// Adds a tilemap entry that refers to already loaded tiles in a tileset.
    /// Returns the index of the map
    pub fn add_tilemap(
        &mut self,
        bank_id: u8,
        tileset_id: TilesetID,
        columns: u16,
        data: &[Cell],
    ) -> MapID {
        // let bank = &mut self.banks[bank_id as usize];
        // let tileset = roster.tilesets[tileset_id.0 as usize];
        let roster = &mut self.assets;

        if roster.map_head as usize >= roster.maps.len() {
            panic!(err!("Map capacity exceeded on bank {}"), bank_id);
        }

        // Add metadata
        let map_idx = roster.map_head;
        let data_start = roster.cell_head;
        let data_len = u16::try_from(data.len()).unwrap();
        let rows = data_len / columns;

        assert!(
            data_len % columns == 0,
            err!("Invalid Tilemap dimensions, data.len() must be divisible by columns")
        );

        // Map entry
        roster.maps[roster.map_head as usize] = Tilemap { bank_id, columns, rows, data_start, data_len };

        // Acquire tile offset for desired tileset
        let tileset = &roster.tilesets[tileset_id.0 as usize];
        let tileset_offset = tileset.tile_start;

        // Add tile entries, mapping the original tile ids to the current tile bank positions
        for (i, &cell) in data.iter().enumerate() {
            roster.cells[data_start as usize + i] =
                Cell { id: TileID(cell.id.0 + tileset_offset), ..cell };
        }

        // Advance and return
        roster.map_head += 1;
        MapID(map_idx)
    }

    // /// Adds an animation entry
    // /// Returns the index of the animation
    // pub fn add_anim<const LEN: usize>(
    //     &mut self,
    //     tileset_id: TilesetID,
    //     fps: u8,
    //     columns: u8,
    //     data: &[Cell],
    // ) -> Option<AnimID> {
    //     if self.anim_head as usize >= self.anims.len() {
    //         return None;
    //     }

    //     // Add metadata
    //     let anim_idx = self.anim_head;
    //     let data_start = self.tile_entry_head;
    //     let data_len = u16::try_from(data.len()).ok()?;
    //     self.anims[self.anim_head as usize] = Anim { fps, columns, data_start, data_len };

    //     // Acquire tile offset for desired tileset
    //     let tileset = &self.tileset_entries[tileset_id.0 as usize];
    //     let tileset_offset = tileset.tile_start;

    //     // Add tile entries, mapping the original tile ids to the current tile bank positions
    //     for (i, &entry) in data.iter().enumerate() {
    //         self.bg.data[data_start as usize + i] =
    //             Cell { id: TileID(entry.id.0 + tileset_offset), ..entry };
    //     }

    //     // Advance and return
    //     self.anim_head += 1;
    //     Some(AnimID(anim_idx))
    // }

    // // Function to add a complete tileset at once
    // // This takes slices of pre-existing data instead of owned structures
    // pub fn add_tileset(
    //     &mut self,
    //     tiles: &[Tile<2>],
    //     animations: &[(u8, &[u16])],  // (fps, frames)
    //     maps: &[(u16, &[Cell])], // (columns, tile_entries)
    // ) -> Option<u16> {
    //     // Record starting positions for everything
    //     let id = self.tileset_head;
    //     let tile_start = self.tile_head as u16;
    //     let anims_start = self.anim_head as u16;
    //     let maps_start = self.map_head as u16;
    //     //

    //     // Add tiles
    //     self.add_tiles(tiles)?;

    //     // Add animations
    //     for &(fps, frames) in animations {
    //         let frames_start = self.add_anim_frames(frames)?;
    //         self.add_animation(fps, frames_start, frames.len() as u16)?;
    //     }

    //     // Add maps
    //     for &(columns, tile_entries) in maps {
    //         let entries_start = self.add_tile_entries(tile_entries)?;
    //         self.add_map(columns, entries_start, tile_entries.len() as u16)?;
    //     }

    //     self.tileset_entries[id as usize] = Tileset {
    //         tile_start,
    //         tiles_count: todo!(),
    //         anims_start,
    //         anims_count: todo!(),
    //         maps_start,
    //         maps_count: todo!(),
    //     };

    //     self.tileset_head += 1;
    //     Some(id)
    // }

    // Retrieval functions

    // // Get a reference to a tileset entry
    // pub fn get_tileset(&self, tileset_idx: usize) -> Option<&Tileset> {
    //     if tileset_idx < self.tileset_head { Some(&self.tileset_entries[tileset_idx]) } else { None }
    // }

    // // Get a specific animation within a tileset
    // pub fn get_animation(&self, tileset_idx: usize, anim_offset: usize) -> Option<&Anim> {
    //     let tileset = self.get_tileset(tileset_idx)?;

    //     if anim_offset < tileset.anims_count as usize {
    //         let anim_idx = tileset.anims_start as usize + anim_offset;
    //         Some(&self.anims[anim_idx])
    //     } else {
    //         None
    //     }
    // }

    // // Get a specific map within a tileset
    // pub fn get_map(&self, tileset_idx: usize, map_offset: usize) -> Option<&MapEntry> {
    //     let tileset = self.get_tileset(tileset_idx)?;

    //     if map_offset < tileset.maps_count as usize {
    //         let map_idx = tileset.maps_start as usize + map_offset;
    //         Some(&self.maps[map_idx])
    //     } else {
    //         None
    //     }
    // }

    // // Get a specific tile within a tileset
    // pub fn get_tile(&self, tileset_idx: usize, tile_offset: usize) -> Option<&Tile<2>> {
    //     let tileset = self.get_tileset(tileset_idx)?;

    //     if tile_offset < tileset.tiles_count as usize {
    //         let tile_idx = tileset.tile_start as usize + tile_offset;
    //         Some(&self.tiles[tile_idx])
    //     } else {
    //         None
    //     }
    // }
}
