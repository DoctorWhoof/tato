use arena::ArenaOps;

use crate::prelude::*;
use core::array::from_fn;

mod anim;
pub use anim::*;

mod color;
use color::*;

mod tileset;
pub use tileset::*;

mod tilemap;
pub use tilemap::*;

/// Allows loading tilesets and their associated assets like Tilemaps and Animations.
/// The tileset's tiles and colors are stored in a memory bank, while the assets
/// are kept internally. All asset's tile indices are remapped to match the actual
/// tile indices currently in the memory bank.
#[derive(Debug)]
pub struct Assets<const CAP: usize> {
    // Main storage
    pub arena: tato_arena::Arena<CAP, u16>,
    // Everything that needs to be counted.
    cell_head: u16,
    tileset_head: u8,
    strip_head: u8,
    map_head: u8,
    color_head: u8,
    anim_head: u8,
    // Asset types
    pub(crate) tilesets: [Tileset; 256],
    pub(crate) map_entries: [TilemapEntry; 256],
    pub(crate) strip_entries: [StripEntry; 256],
    pub(crate) anim_entries: [AnimEntry; 256],
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
    strip_head: u8,
    map_head: u8,
    color_head: u8,
    anim_head: u8,
    // Bank states (tile and palette counts)
    bank_tile_counts: [u8; BANK_COUNT],
    bank_color_mapping_counts: [u8; BANK_COUNT],
    // bank_palette_counts: [u8; BANK_COUNT],
}

impl<const CAP: usize> Assets<CAP> {
    pub fn new() -> Self {
        Self {
            arena: tato_arena::Arena::new(),
            // Metadata
            tilesets: from_fn(|_| Tileset::default()),
            map_entries: from_fn(|_| TilemapEntry::default()),
            strip_entries: from_fn(|_| StripEntry::default()),
            anim_entries: from_fn(|_| AnimEntry::default()),
            // Counters
            cell_head: 0,
            tileset_head: 0,
            strip_head: 0,
            map_head: 0,
            color_head: 0,
            anim_head: 0,
            // Checkpoint system
            checkpoints: from_fn(|_| TilesetCheckpoint::default()),
            checkpoint_head: 0,
        }
    }

    pub fn reset(&mut self) {
        self.cell_head = 0;
        self.tileset_head = 0;
        self.strip_head = 0;
        self.map_head = 0;
        self.anim_head = 0;
        self.color_head = 0;
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
    pub fn push_tile(&mut self, bank_id: u8, tile: &Tile<4>) -> TileID {
        self.banks[bank_id as usize].add_tile(tile)
    }



    /// Adds a tileset as a batch of tiles to the bank
    /// Returns the tileset id.
    pub fn push_tileset(&mut self, bank_id: u8, data: TilesetData) -> TatoResult<TilesetID> {
        // Check if bank_id is valid before storing checkpoint
        if bank_id as usize >= self.banks.len() {
            return Err(TatoError::InvalidBankId(bank_id));
        }

        // Save checkpoint before loading
        {
            let assets = &mut self.assets;
            if assets.checkpoint_head >= 32 {
                return Err(TatoError::CheckpointStackOverflow);
            }

            // Save bank states
            let mut bank_tile_counts = [0u8; BANK_COUNT];
            let mut bank_palette_counts = [0u8; BANK_COUNT];
            let mut bank_color_mapping_counts = [0u8; BANK_COUNT];
            for (i, bank) in self.banks.iter().enumerate().take(BANK_COUNT) {
                bank_tile_counts[i] = bank.tile_count() as u8;
                bank_palette_counts[i] = bank.color_count();
                bank_color_mapping_counts[i] = bank.color_mapping_count();
            }

            assets.checkpoints[assets.checkpoint_head as usize] = TilesetCheckpoint {
                arena_offset: assets.arena.used() as u16,
                cell_head: assets.cell_head,
                tileset_head: assets.tileset_head,
                strip_head: assets.strip_head,
                map_head: assets.map_head,
                color_head: assets.color_head,
                anim_head: assets.anim_head,
                bank_tile_counts,
                bank_color_mapping_counts,
            };

            assets.checkpoint_head += 1;
        }

        let bank = match self.banks.get_mut(bank_id as usize) {
            Some(bank) => bank,
            None => {
                // Rollback checkpoint
                self.assets.checkpoint_head -= 1;
                return Err(TatoError::InvalidBankId(bank_id));
            },
        };

        let assets = &mut self.assets;
        let tiles_len = data.tiles.map_or(0, |tiles| tiles.len());
        if bank.tile_count() + tiles_len > bank.tile_capacity() {
            // Rollback checkpoint
            assets.checkpoint_head -= 1;
            return Err(TatoError::TilesetCapacityExceeded {
                bank_id,
                requested: tiles_len,
                available: bank.tile_capacity() - bank.tile_count(),
            });
        }
        let id = assets.tileset_head;

        // Tile processing
        let tile_start = u8::try_from(bank.tile_count()).unwrap();
        if let Some(tiles) = data.tiles {
            for tile in tiles.iter() {
                bank.add_tile(tile);
            }
        }

        // Color mapping processing
        // Track the actual bank indices where each mapping is stored
        let mut mapping_indices = [0u8; COLOR_MAPPING_COUNT as usize]; // Max 16 mappings
        let color_mapping_count = if let Some(mappings) = data.color_mappings {
            // First mapping is always identity at index 0
            mapping_indices[0] = 0;

            if mappings.len() > 1 {
                // Push remaining mappings and track their actual indices
                for (i, mapping) in mappings[1..].iter().enumerate() {
                    let bank_index = bank.push_color_mapping(*mapping);
                    mapping_indices[i + 1] = bank_index;
                }
                mappings.len() as u8
            } else {
                1 // Just identity mapping
            }
        } else {
            1 // Default to identity only
        };

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

        // Build tileset entry
        assets.tilesets[id as usize] = Tileset {
            bank_id,
            tile_start,
            colors: tileset_colors,
            color_count,
            color_mapping_indices: mapping_indices,
            color_mapping_count,
        };
        assets.color_head += color_count;
        assets.tileset_head += 1;
        Ok(TilesetID(id))
    }

    /// Restore to previous checkpoint, unloading the last tileset
    pub fn pop_tileset(&mut self) -> TatoResult<()> {
        let assets = &mut self.assets;
        if assets.checkpoint_head == 0 {
            return Err(TatoError::NoTilesetToPop);
        }

        assets.checkpoint_head -= 1;
        let checkpoint = assets.checkpoints[assets.checkpoint_head as usize];

        // Restore arena state
        assets.arena.restore_to(checkpoint.arena_offset as usize);

        // Clear animation and tilemap entries that were created after the checkpoint
        for i in checkpoint.strip_head..assets.strip_head {
            assets.strip_entries[i as usize] = StripEntry::default();
        }
        for i in checkpoint.map_head..assets.map_head {
            assets.map_entries[i as usize] = TilemapEntry::default();
        }
        for i in checkpoint.anim_head..assets.anim_head {
            assets.anim_entries[i as usize] = AnimEntry::default();
        }

        // Restore all counters
        assets.cell_head = checkpoint.cell_head;
        assets.tileset_head = checkpoint.tileset_head;
        assets.strip_head = checkpoint.strip_head;
        assets.map_head = checkpoint.map_head;
        assets.color_head = checkpoint.color_head;
        assets.anim_head = checkpoint.anim_head;

        // Restore bank state
        for (i, bank) in self.banks.iter_mut().enumerate().take(BANK_COUNT) {
            bank.restore_tile_count(checkpoint.bank_tile_counts[i]);
            bank.restore_palette_state(
                bank.color_count(), // Keep current color count
                checkpoint.bank_color_mapping_counts[i],
            );
        }
        Ok(())
    }

    pub fn load_tilemap<const LEN: usize>(
        &mut self,
        tileset_id: TilesetID,
        map: &Tilemap<LEN>,
    ) -> TatoResult<MapID> {
        // Validate that maps can only be loaded for the current tileset
        let expected = self.assets.tileset_head.saturating_sub(1);
        if tileset_id.0 != expected {
            // TODO: I ran into tis error and it was... confusing.
            // Maybe a simple panic is better, provides call stack, etc.
            return Err(TatoError::InvalidTilesetForMap { provided: tileset_id.0, expected });
        }

        // Acquire tile offset for desired tileset
        let tileset = self
            .assets
            .tilesets
            .get(tileset_id.0 as usize)
            .ok_or(TatoError::InvalidTilesetId(tileset_id.0))?;
        let tileset_offset = tileset.tile_start;
        let bank_id = tileset.bank_id;

        if self.assets.map_head as usize >= self.assets.map_entries.len() {
            return Err(TatoError::MapCapacityExceeded { bank_id });
        }

        if map.len() % map.columns() as usize != 0 {
            return Err(TatoError::InvalidTilemapDimensions {
                len: map.len(),
                columns: map.columns(),
            });
        }

        // Allocate remapped cells in arena
        let cells_slice = self
            .assets
            .arena
            .alloc_slice_from_fn(map.len(), |i| {
                let cell = &map.cells()[i];
                Cell {
                    id: TileID(cell.id.0 + tileset_offset),
                    flags: cell.flags,
                    // Translate color_mapping index using the mapping table
                    color_mapping: if (cell.color_mapping as usize) < tileset.color_mapping_count as usize {
                        tileset.color_mapping_indices[cell.color_mapping as usize]
                    } else {
                        0 // Fallback to identity if index is out of bounds
                    },
                    group: cell.group,
                }
            })
            .map_err(TatoError::Arena)?;

        // Store entry
        let map_idx = self.assets.map_head;
        self.assets.map_entries[map_idx as usize] =
            TilemapEntry { cells: cells_slice, columns: map.columns(), rows: map.rows() };

        if self.assets.map_head == 255 {
            return Err(TatoError::TilemapCapacityReached);
        } else {
            self.assets.map_head += 1;
            Ok(MapID(map_idx))
        }
    }

    pub fn load_animation_strip<const FRAME_LEN: usize, const FRAME_COUNT: usize>(
        &mut self,
        tileset_id: TilesetID,
        frames: &[Tilemap<FRAME_LEN>; FRAME_COUNT],
    ) -> TatoResult<StripID> {
        // Acquire tile offset for desired tileset
        let frames_idx = self.assets.strip_head;
        let start_index = self.assets.map_head;
        let frame_count =
            u8::try_from(FRAME_COUNT).map_err(|_| TatoError::FrameCountTooLarge(FRAME_COUNT))?;
        let available = 255 - start_index;
        if available < frame_count {
            return Err(TatoError::InsufficientAnimationFrameSpace {
                requested: FRAME_COUNT,
                available,
            });
        }
        // Load frames
        for map in frames {
            self.load_tilemap(tileset_id, map)?;
        }
        // Store entry
        self.assets.strip_entries[frames_idx as usize] = StripEntry { start_index, frame_count };
        // Advance and return
        if self.assets.strip_head == 255 {
            return Err(TatoError::AnimationFramesCapacityExceeded);
        } else {
            self.assets.strip_head += 1;
            Ok(StripID(frames_idx))
        }
    }

    pub fn init_anim<const LEN: usize>(&mut self, anim: Anim<LEN>) -> TatoResult<AnimID> {
        if self.assets.anim_head == 255 {
            return Err(TatoError::AnimationCapacityReached);
        }

        let strip = self
            .assets
            .strip_entries
            .get(anim.strip.0 as usize)
            .ok_or(TatoError::InvalidStripId(anim.strip.0))?;

        // Reserve index 0 for "no animation", start allocation from index 1
        let next_index = if self.assets.anim_head == 0 { 1 } else { self.assets.anim_head + 1 };

        // Check capacity
        if next_index as usize >= self.assets.anim_entries.len() {
            return Err(TatoError::AnimationCapacityExceeded);
        }

        // Validate
        for frame in &anim.frames {
            if *frame >= strip.frame_count {
                return Err(TatoError::InvalidFrameIndex {
                    frame: *frame,
                    max_frames: strip.frame_count,
                });
            }
        }

        let frames = self
            .assets
            .arena
            .alloc_slice_from_fn(anim.frames.len(), |i| anim.frames[i])
            .map_err(TatoError::Arena)?;
        self.assets.anim_entries[next_index as usize] =
            AnimEntry { frames, fps: anim.fps, rep: anim.rep, strip: anim.strip };

        // Update head to track the last allocated index
        self.assets.anim_head = next_index;
        Ok(AnimID(next_index))
    }

    pub fn get_tilemap(&self, map_id: MapID) -> TatoResult<TilemapRef<'_>> {
        let entry = self
            .assets
            .map_entries
            .get(map_id.0 as usize)
            .ok_or(TatoError::InvalidMapId(map_id.0))?;
        let cells =
            self.assets.arena.get_slice(entry.cells).map_err(TatoError::Arena)?;
        Ok(TilemapRef { cells, columns: entry.columns, rows: entry.rows })
    }

    pub fn get_anim_frame_tilemap(&self, anim_id: AnimID, frame_index: usize) -> TatoResult<TilemapRef<'_>> {
        // Get the animation entry
        let anim_entry = self
            .assets
            .anim_entries
            .get(anim_id.0 as usize)
            .ok_or(TatoError::InvalidAnimId(anim_id.0))?;

        // Get the frame value from the animation's frame array
        let frames = self.assets.arena.get_slice(anim_entry.frames).map_err(TatoError::Arena)?;
        let frame_value = frames
            .get(frame_index)
            .ok_or(TatoError::InvalidFrameIndex {
                frame: frame_index as u8,
                max_frames: frames.len() as u8,
            })?;

        // Get the strip entry to find where the tilemaps are stored
        let strip_entry = self
            .assets
            .strip_entries
            .get(anim_entry.strip.0 as usize)
            .ok_or(TatoError::InvalidStripId(anim_entry.strip.0))?;

        // Calculate the actual tilemap index
        let map_index = strip_entry.start_index + frame_value;

        // Get the tilemap
        self.get_tilemap(MapID(map_index))
    }

    /// Gets the tilemap for the current frame of an animation based on the video chip's
    /// internal frame counter.
    pub fn get_current_anim_tilemap(&self, anim_id: AnimID) -> TatoResult<TilemapRef<'_>> {
        // AnimID(0) means no animation
        if anim_id.0 == 0 {
            return Err(TatoError::InvalidAnimId(0));
        }

        // Calculate current frame index based on animation timing
        // This uses the public get_anim_frame method from graphics.rs
        let frame_index = self.get_anim_frame(anim_id);

        // Get the tilemap for that frame
        self.get_anim_frame_tilemap(anim_id, frame_index)
    }

    // pub fn get_animation(&self, anim_id: AnimID) -> AnimRef {
    //     let entry = &self.assets.anim_entries[anim_id.0 as usize];
    //     AnimRef {
    //         frames: self.assets.arena.get_slice(&entry.frames),
    //         fps: entry.fps,
    //         rep: entry.rep,
    //         // tileset: entry.tileset,
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tato_video::SpriteBundle;

    #[test]
    fn test_anim_id_zero_is_no_animation() {
        // Test that AnimID::default() returns AnimID(0)
        let default_anim = AnimID::default();
        assert_eq!(default_anim.0, 0);

        // Test that get_anim_frame handles AnimID(0) correctly
        let mut tato = crate::Tato::new(160, 144, 60);
        let frame = tato.get_anim_frame(default_anim);
        assert_eq!(frame, 0); // Should return 0 for "no animation"

        // Test that draw_anim_to_fg handles AnimID(0) gracefully (no panic)
        let bundle = SpriteBundle { x: 0, y: 0, flip_x: false, flip_y: false };
        tato.draw_anim_to_fg(default_anim, bundle);
        // If we reach here without panic, the test passes
    }

    #[test]
    fn test_first_anim_id_is_one() {
        // Test that animation allocation starts from 1, not 0
        let mut assets = Assets::<1024>::new();

        // Simulate what init_anim does for allocation
        let next_index = if assets.anim_head == 0 { 1 } else { assets.anim_head + 1 };
        assert_eq!(next_index, 1); // First animation should get ID 1

        // Verify that after "allocation", head is updated correctly
        assets.anim_head = next_index;
        assert_eq!(assets.anim_head, 1);
    }

    #[test]
    fn test_get_anim_frame_tilemap() {
        let tato = crate::Tato::new(160, 144, 60);

        // Test that AnimID(0) returns error for get_current_anim_tilemap
        let no_anim = AnimID(0);
        let result = tato.get_current_anim_tilemap(no_anim);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TatoError::InvalidAnimId(0));

        // Test that uninitialized AnimID returns error
        // Note: We use ID 1 which is valid range but uninitialized
        let uninit_anim = AnimID(1);
        let result = tato.get_anim_frame_tilemap(uninit_anim, 0);
        assert!(result.is_err());
    }
}
