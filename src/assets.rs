use crate::*;
use core::array::from_fn;

// mod palette;
// use palette::*;

mod anim;
pub use anim::*;

mod tileset;
pub use tileset::*;

mod tilemap;
pub use tilemap::*;

/// Stores metadata associating assets (Tilemaps, Animations and Fonts) to a
/// tileset and its tiles currently loaded in a video memory bank
#[derive(Debug, Clone)]
pub struct Assets {
    pub tilesets: [Tileset; 256],
    // Asset types
    pub anims: [Anim; 256],
    pub maps: [Tilemap; 256],
    // pub palettes: [Palette; 256],
    // pub fonts: [Font; 256],
    // "flat" storage for cells used by any asset type.
    pub cells: [Cell; 2048],

    pub colors: [Color12Bit; 256],
    pub sub_palettes: [[u8; 4]; 256],
    // Everything that needs to be counted.
    cell_head: u16,
    tileset_head: u8,
    anim_head: u8,
    map_head: u8,
    color_head: u8,
    sub_palette_head: u8,
}

impl Assets {
    pub fn new() -> Self {
        Self {
            // Metadata
            tilesets: from_fn(|_| Tileset::default()),
            anims: from_fn(|_| Anim::default()),
            maps: from_fn(|_| Tilemap::default()),
            colors: from_fn(|_| Color12Bit::default()),
            sub_palettes: from_fn(|_| Default::default()),
            // "Flat" entry data for maps, anims and fonts
            cells: from_fn(|_| Cell::default()),

            // Counters
            cell_head: 0,
            tileset_head: 0,
            anim_head: 0,
            map_head: 0,
            color_head: 0,
            sub_palette_head: 0,
        }
    }

    pub fn reset(&mut self) {
        self.cell_head = 0;
        self.tileset_head = 0;
        self.anim_head = 0;
        self.map_head = 0;
        self.color_head = 0;
        self.sub_palette_head = 0;
    }
}

impl Tato {
    /// Adds a single tile, returns a TileID
    #[inline]
    pub fn new_tile(&mut self, bank_id: u8, tile: &Tile<2>) -> TileID {
        self.banks[bank_id as usize].add_tile(tile)
    }

    /// Adds a tileset as a batch of tiles to the bank
    /// Returns the tileset id.
    pub fn new_tileset(&mut self, bank_id: u8, data: TilesetData) -> Option<TilesetID> {
        let bank = self.banks.get_mut(bank_id as usize)?;
        let roster = &mut self.assets;
        if bank.tile_count() + data.tiles.len() > bank.tile_capacity() {
            return None;
        }
        let id = roster.tileset_head;

        // Tile processing
        let tile_start = u16::try_from(bank.tile_count()).unwrap();
        let tiles_count = u16::try_from(data.tiles.len()).unwrap();

        for tile in data.tiles.iter() {
            bank.add_tile(tile);
        }

        // Color processing
        if let Some(colors) = data.colors {
            let colors_start = roster.color_head;
            let colors_len = u8::try_from(colors.len()).unwrap();

            for (i, color) in colors.iter().enumerate() {
                let index = colors_start as usize + i;
                roster.colors[index] = *color;
                bank.palette[index] = *color;
            }
            // TODO: Needs to "build" a VideoMemory palette from smaller palettes,
            // skip redundant colors
            // let mut bg_palette = Palette::default();
            // for color in data.palette {
            //     // remove colors already in use
            //     if !self.video.bg_palette.contains(color) {
            //         bg_palette.push(*color);
            //     }
            // }

            // let palette_id = PaletteID(roster.palettte_head);
            // roster.tilesets[id as usize] = Tileset { palette_id, bank_id, tile_start, tiles_count };
            roster.tilesets[id as usize] =
                Tileset { bank_id, tile_start, tiles_count, colors_start, colors_len };
        } else {
            roster.tilesets[id as usize] = Tileset {
                bank_id,
                tile_start,
                tiles_count,
                colors_start: roster.color_head,
                colors_len: 0,
            };
        }

        roster.color_head += 1;
        roster.tileset_head += 1;
        Some(TilesetID(id))
    }

    // TODO: Make private, loading tilesets should load all associated assets

    /// Adds a tilemap entry that refers to already loaded tiles in a tileset.
    /// Returns the index of the map
    pub fn new_tilemap(&mut self, tileset_id: TilesetID, columns: u16, data: &[Cell]) -> MapID {
        // let bank = &mut self.banks[bank_id as usize];
        // Acquire tile offset for desired tileset
        let roster = &mut self.assets;
        let tileset = &roster.tilesets[tileset_id.0 as usize];
        let tileset_offset = tileset.tile_start;
        let bank_id = tileset.bank_id;

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
        roster.maps[roster.map_head as usize] =
            Tilemap { bank_id, columns, rows, data_start, data_len };

        // Add tile entries, mapping the original tile ids to the current tile bank positions
        for (i, &cell) in data.iter().enumerate() {
            roster.cells[data_start as usize + i] = Cell {
                id: TileID(cell.id.0 + tileset_offset), //
                flags: cell.flags,
            };
        }

        // Advance and return
        roster.map_head += 1;
        roster.cell_head += data_len;
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
}
