use crate::*;
use core::array::from_fn;

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
    pub anims: [AnimEntry; 256],
    pub maps: [TilemapEntry; 256],
    // pub palettes: [Palette; 256],
    // pub fonts: [Font; 256],
    // "flat" storage for cells used by any asset type.
    pub cells: [Cell; 2048],

    pub colors: [RGBA12; 256],
    pub sub_palettes: [[u8; 4]; 256],
    // Everything that needs to be counted.
    pub(crate) cell_head: u16,
    pub(crate) tileset_head: u8,
    pub(crate) anim_head: u8,
    pub(crate) map_head: u8,
    pub(crate) color_head: u8,
    pub(crate) sub_palette_head: u8,
}

impl Assets {
    pub fn new() -> Self {
        Self {
            // Metadata
            tilesets: from_fn(|_| Tileset::default()),
            anims: from_fn(|_| AnimEntry::default()),
            maps: from_fn(|_| TilemapEntry::default()),
            colors: from_fn(|_| RGBA12::default()),
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

    pub fn new_subpalette(
        &mut self,
        bank_id: u8,
        sub_palette: [ColorID; COLORS_PER_TILE as usize],
    ) -> PaletteID {
        let bank = self.banks.get_mut(bank_id as usize).unwrap();
        let assets = &mut self.assets;
        let palette_id = assets.sub_palette_head;
        bank.push_subpalette(sub_palette);
        assets.sub_palette_head += 1;
        PaletteID(palette_id)
    }

    /// Adds a tileset as a batch of tiles to the bank
    /// Returns the tileset id.
    pub fn new_tileset(&mut self, bank_id: u8, data: TilesetData) -> Option<TilesetID> {
        let bank = self.banks.get_mut(bank_id as usize)?;
        let assets = &mut self.assets;
        if bank.tile_count() + data.tiles.len() > bank.tile_capacity() {
            return None;
        }
        let id = assets.tileset_head;

        // Tile processing
        let tile_start = u8::try_from(bank.tile_count()).unwrap();
        let tiles_count = u8::try_from(data.tiles.len()).unwrap();

        for tile in data.tiles.iter() {
            bank.add_tile(tile);
        }

        // Main Color processing
        let mut color_entries: [ColorEntry; COLORS_PER_PALETTE as usize] = Default::default();
        let mut color_count = 0;
        let colors_start = assets.color_head;

        // 'color_process:{
        if let Some(data_colors) = data.colors {
            for (i, color) in data_colors.iter().enumerate() {
                let mut reused_color = false;
                let mut index = colors_start;
                // Compare to bank colors
                'color_check: for slot in 0..bank.color_count() {
                    let bank_color = bank.palette[slot as usize];
                    if *color == bank_color {
                        reused_color = true;
                        index = slot;
                        break 'color_check;
                    }
                }
                if !reused_color {
                    index = colors_start + color_count;
                    // if index >= COLORS_PER_PALETTE {
                    //     // TODO: Should it panic? Result? Need to think about it.
                    //     break 'color_process;
                    // }
                    // Immediately also set the color in the bank
                    bank.push_color(*color);
                    // Increment count since we added a new one
                    color_count += 1;
                    println!("{}: {}", index, *color);
                } else {
                }
                // Store color entry for management
                color_entries[i] = ColorEntry { reused_color, index, value: *color };
            }
        }
        // }

        // Sub palette processing
        // Maps indices starting at zero to the actual current color positions in the bank
        let sub_palettes_start = bank.sub_palette_count();
        let mut sub_palettes_len = 0;
        if let Some(sub_palettes) = data.sub_palettes {
            for sub_palette in sub_palettes {
                let mapped_sub_palette: [ColorID; COLORS_PER_TILE as usize] = from_fn(|i| {
                    let mapped = color_entries[sub_palette[i] as usize].index;
                    ColorID(mapped)
                });
                bank.push_subpalette(mapped_sub_palette);
                sub_palettes_len += 1;
            }
        }

        // Build tileset entry
        assets.tilesets[id as usize] = Tileset {
            bank_id,
            tile_start,
            tiles_count,
            color_entries,
            color_count,
            sub_palettes_start,
            sub_palettes_len,
        };

        assets.color_head += color_count;
        assets.sub_palette_head += sub_palettes_len;
        assets.tileset_head += 1;
        Some(TilesetID(id))
    }

    // TODO: Make private, loading tilesets should load all associated assets?
    // In that case, how would I obtain the MapID? Doesn't seem to work...

    /// Adds a tilemap entry that refers to already loaded tiles in a tileset.
    /// Returns the index of the map
    pub fn new_tilemap<const LEN: usize>(
        &mut self,
        tileset_id: TilesetID,
        data: &BGMap<LEN>,
    ) -> MapID {
        // Acquire tile offset for desired tileset
        let assets = &mut self.assets;
        let tileset = &assets.tilesets[tileset_id.0 as usize];
        let tileset_offset = tileset.tile_start;
        let bank_id = tileset.bank_id;

        if assets.map_head as usize >= assets.maps.len() {
            panic!(err!("Map capacity exceeded on bank {}"), bank_id);
        }

        // Add metadata
        let map_idx = assets.map_head;
        let data_start = assets.cell_head;
        let data_len = u16::try_from(data.len()).unwrap();
        let columns = data.columns;
        let rows = data_len / data.columns;

        assert!(
            data_len % data.columns == 0,
            err!("Invalid Tilemap dimensions, data.len() must be divisible by columns")
        );

        // Map entry
        assets.maps[assets.map_head as usize] =
            TilemapEntry { bank_id, columns, rows, data_start, data_len };

        // Add tile entries, mapping the original tile ids to the current tile bank positions
        for (i, &cell) in data.cells.iter().enumerate() {
            let mut flags = cell.flags;
            flags.set_palette(PaletteID(cell.flags.palette().0 + tileset.sub_palettes_start));
            assets.cells[data_start as usize + i] = Cell {
                id: TileID(cell.id.0 + tileset_offset), //
                flags,
            };
        }

        // Advance and return
        assets.map_head += 1;
        assets.cell_head += data_len;
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

    // pub fn get_tilemap(&self, id: MapID) -> Option<Tilemap> {
    //     if id.0 >= self.assets.map_head {
    //         return None;
    //     }

    //     let map = &self.assets.maps[id.0 as usize];
    //     let start = map.data_start as usize;
    //     let end = start + map.data_len as usize;
    //     let cells = &self.assets.cells[start..end];

    //     Some(Tilemap { cells, columns: map.columns, rows: map.rows })
    // }
}
