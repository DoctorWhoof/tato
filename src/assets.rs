use crate::prelude::*;
use core::array::from_fn;

mod anim;
pub use anim::*;

mod color_entry;
use color_entry::*;

mod tileset;
pub use tileset::*;

mod tilemap_entry;
pub use tilemap_entry::*;

// mod tilemap_ref;
// pub use tilemap_ref::*;

/// Stores metadata associating assets (Tilemaps, Animations and Fonts) to a
/// tileset and its tiles currently loaded in a video memory bank
#[derive(Debug)]
pub struct Assets<const CAP: usize> {
    // Main storage
    arena: tato_arena::Arena<CAP, u16>,
    // Everything that needs to be counted.
    cell_head: u16,
    tileset_head: u8,
    anim_head: u8,
    map_head: u8,
    color_head: u8,
    sub_palette_head: u8,
    // Asset types
    pub(crate) tilesets: [Tileset; 256],
    map_entries: [TilemapEntry; 256],
    // Checkpoint system
    checkpoints: [TilesetCheckpoint; 32],
    checkpoint_head: u8,
}

/// Checkpoint for stack-based tileset management
#[derive(Debug, Clone, Copy, Default)]
struct TilesetCheckpoint {
    // This Arena caches tilemaps with remapped values (i.e. they start
    // with an offset that matches the data loaded in the video bank,
    // instead of starting from zero).
    arena_offset: u16,
    // Asset counters
    cell_head: u16,
    tileset_head: u8,
    anim_head: u8,
    map_head: u8,
    color_head: u8,
    sub_palette_head: u8,
    // Bank states (tile and palette counts)
    bank_tile_counts: [u8; TILE_BANK_COUNT],
    bank_palette_counts: [u8; TILE_BANK_COUNT],
    bank_sub_palette_counts: [u8; TILE_BANK_COUNT],
}

impl<const CAP: usize> Assets<CAP> {
    pub fn new() -> Self {
        Self {
            arena: tato_arena::Arena::new(),
            // Metadata
            tilesets: from_fn(|_| Tileset::default()),
            map_entries: from_fn(|_| TilemapEntry::default()),
            // Counters
            cell_head: 0,
            tileset_head: 0,
            anim_head: 0,
            map_head: 0,
            color_head: 0,
            sub_palette_head: 0,
            // Arena and checkpoint system
            // anims: from_fn(|_| Pool::default()),
            checkpoints: from_fn(|_| TilesetCheckpoint::default()),
            checkpoint_head: 0,
        }
    }

    pub fn reset(&mut self) {
        self.cell_head = 0;
        self.tileset_head = 0;
        self.anim_head = 0;
        self.map_head = 0;
        self.color_head = 0;
        self.sub_palette_head = 0;
        self.arena.clear();
        self.checkpoint_head = 0;
    }

    pub fn used_memory(&self) -> usize {
        self.arena.used()
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
    pub fn push_tileset(&mut self, bank_id: u8, data: TilesetData) -> Option<TilesetID> {
        // Check if bank_id is valid before storing checkpoint
        if bank_id as usize >= self.banks.len() {
            return None;
        }

        // Save checkpoint before loading
        {
            let assets = &mut self.assets;
            assert!(assets.checkpoint_head < 32, "Checkpoint stack overflow (max 32 tilesets)");

            // Save bank states
            let mut bank_tile_counts = [0u8; TILE_BANK_COUNT];
            let mut bank_palette_counts = [0u8; TILE_BANK_COUNT];
            let mut bank_sub_palette_counts = [0u8; TILE_BANK_COUNT];
            for (i, bank) in self.banks.iter().enumerate().take(TILE_BANK_COUNT) {
                bank_tile_counts[i] = bank.tile_count() as u8;
                bank_palette_counts[i] = bank.color_count();
                bank_sub_palette_counts[i] = bank.sub_palette_count();
            }

            assets.checkpoints[assets.checkpoint_head as usize] = TilesetCheckpoint {
                arena_offset: assets.arena.used() as u16,
                cell_head: assets.cell_head,
                tileset_head: assets.tileset_head,
                anim_head: assets.anim_head,
                map_head: assets.map_head,
                color_head: assets.color_head,
                sub_palette_head: assets.sub_palette_head,
                bank_tile_counts,
                bank_palette_counts,
                bank_sub_palette_counts,
            };

            assets.checkpoint_head += 1;
        }

        let bank = match self.banks.get_mut(bank_id as usize) {
            Some(bank) => bank,
            None => {
                // Rollback checkpoint
                self.assets.checkpoint_head -= 1;
                return None;
            },
        };
        let assets = &mut self.assets;
        if bank.tile_count() + data.tiles.len() > bank.tile_capacity() {
            // Rollback checkpoint
            assets.checkpoint_head -= 1;
            return None;
        }
        let id = assets.tileset_head;

        // Tile processing
        let tile_start = u8::try_from(bank.tile_count()).unwrap();
        // let tiles_count = u8::try_from(data.tiles.len()).unwrap();

        for tile in data.tiles.iter() {
            bank.add_tile(tile);
        }

        // Main Color processing
        let mut color_entries: [ColorEntry; COLORS_PER_PALETTE as usize] = Default::default();
        let mut tileset_colors = [RGBA12::default(); COLORS_PER_PALETTE as usize];
        let mut color_count = 0;
        let colors_start = assets.color_head;

        if let Some(data_colors) = data.colors {
            for (i, color) in data_colors.iter().enumerate() {
                // Copy to tileset colors array
                if i < 256 {
                    tileset_colors[i] = *color;
                }

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
                    // Immediately also set the color in the bank
                    index = bank.push_color(*color).0;
                    // Increment count since we added a new one
                    color_count += 1;
                }
                // Store color entry for management
                color_entries[i] = ColorEntry { reused_color, index, value: *color };
            }
        }

        // Sub palette processing
        let sub_palettes_start = bank.sub_palette_count();
        let mut sub_palettes_len = 0;
        let mut tileset_sub_palettes = [[0u8; 4]; SUBPALETTE_COUNT as usize]; // Initialize tileset sub_palettes array

        if let Some(sub_palettes) = data.sub_palettes {
            for (i, sub_palette) in sub_palettes.iter().enumerate() {
                // Copy to tileset sub_palettes array
                if i < 256 {
                    tileset_sub_palettes[i] = **sub_palette;
                }

                let mapped_sub_palette: [ColorID; COLORS_PER_TILE as usize] = from_fn(|j| {
                    let mapped = color_entries[sub_palette[j] as usize].index;
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
            colors: tileset_colors,
            sub_palettes: tileset_sub_palettes,
            color_count,
            sub_palette_count: sub_palettes_len,
            sub_palettes_start,
            sub_palettes_len,
        };
        assets.color_head += color_count;
        assets.sub_palette_head += sub_palettes_len;
        assets.tileset_head += 1;
        Some(TilesetID(id))
    }

    /// Restore to previous checkpoint, unloading the last tileset
    pub fn pop_tileset(&mut self) {
        let assets = &mut self.assets;
        assert!(assets.checkpoint_head > 0, "No tileset to pop");

        assets.checkpoint_head -= 1;
        let checkpoint = assets.checkpoints[assets.checkpoint_head as usize];

        // Restore arena state
        unsafe {
            assets.arena.restore_to(checkpoint.arena_offset as usize);
        }

        // Restore all counters
        assets.cell_head = checkpoint.cell_head;
        assets.tileset_head = checkpoint.tileset_head;
        assets.anim_head = checkpoint.anim_head;
        assets.map_head = checkpoint.map_head;
        assets.color_head = checkpoint.color_head;
        assets.sub_palette_head = checkpoint.sub_palette_head;

        // Restore bank states
        for (i, bank) in self.banks.iter_mut().enumerate().take(TILE_BANK_COUNT) {
            bank.restore_tile_count(checkpoint.bank_tile_counts[i]);
            bank.restore_palette_state(
                checkpoint.bank_palette_counts[i],
                checkpoint.bank_sub_palette_counts[i],
            );
        }
    }

    pub fn load_tilemap<const LEN: usize>(
        &mut self,
        tileset_id: TilesetID,
        map: &Tilemap<LEN>,
    ) -> MapID {
        // Validate that maps can only be loaded for the current tileset
        // let assets = &mut self.assets;
        assert_eq!(
            tileset_id.0,
            self.assets.tileset_head.saturating_sub(1),
            "Can only load maps for the current (most recent) tileset"
        );

        // Acquire tile offset for desired tileset
        let tileset = &self.assets.tilesets[tileset_id.0 as usize];
        let tileset_offset = tileset.tile_start;
        let bank_id = tileset.bank_id;

        if self.assets.map_head as usize >= self.assets.map_entries.len() {
            panic!(err!("Map capacity exceeded on bank {}"), bank_id);
        }

        assert!(
            map.len() % map.columns as usize == 0,
            err!("Invalid Tilemap dimensions, data.len() must be divisible by columns")
        );

        // Allocate remapped cells in arena
        let cells_pool = self
            .assets
            .arena
            .alloc_pool_from_fn(map.len(), |i| {
                let cell = &map.cells[i];
                let mut flags = cell.flags;
                flags.set_palette(PaletteID(cell.flags.palette().0 + tileset.sub_palettes_start));
                Cell { id: TileID(cell.id.0 + tileset_offset), flags }
            })
            .expect("Arena out of space");

        // Store entry
        let map_idx = self.assets.map_head;
        self.assets.map_entries[map_idx as usize] =
            TilemapEntry { cells: cells_pool, columns: map.columns, rows: map.rows };

        self.assets.map_head += 1;
        MapID(map_idx)
    }

    pub fn get_tilemap(&self, map_id: MapID) -> TilemapRef {
        let entry = &self.assets.map_entries[map_id.0 as usize];
        TilemapRef {
            cells: self.assets.arena.get_pool(&entry.cells),
            columns: entry.columns,
            rows: entry.rows,
        }
    }
}
