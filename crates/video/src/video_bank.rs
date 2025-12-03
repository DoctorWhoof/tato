use crate::*;
use core::array::from_fn;

/// A "Memory Bank" that contains the actual tile pixels, a color palette
/// and the subpalettes associated with the main color palette.
#[derive(Debug, Clone)]
pub struct VideoBank<const TILES: usize> {
    pub tiles: [Tile<2>; TILES],
    pub palette: [RGBA12; COLORS_PER_PALETTE as usize],
    pub color_mapping: [[u8; COLORS_PER_PALETTE as usize]; 16],
    // Everything that needs to be counted
    tile_head: u8,
    palette_head: u8,
}

impl<const TILES: usize> VideoBank<TILES> {
    pub fn new() -> Self {
        Self {
            tiles: from_fn(|_| Tile::default()),
            palette: PALETTE_DEFAULT,
            color_mapping: from_fn(|_| from_fn(|i| i as u8)), // default mapping is index=value
            tile_head: 0,
            palette_head: 0,
        }
    }

    pub fn reset(&mut self) {
        // Simply sets internal counters to 0.
        self.tile_head = 0;
        // Will reset colors to their defaults
        self.reset_palettes();
    }

    pub fn reset_palettes(&mut self) {
        self.palette = PALETTE_DEFAULT;
        self.palette_head = 0;
    }

    pub fn push_color(&mut self, color: RGBA12) -> ColorID {
        assert!(self.palette_head < COLORS_PER_PALETTE as u8, "Palette capacity reached");
        let id = ColorID(self.palette_head);
        self.palette[self.palette_head as usize] = color;
        self.palette_head += 1;
        id
    }

    pub fn load_default_colors(&mut self) {
        self.palette = [
            RGBA12::TRANSPARENT, // 0
            RGBA12::BLACK,       // 1
            RGBA12::GRAY,        // 2
            RGBA12::WHITE,       // 3
            RGBA12::DARK_RED,    // 4
            RGBA12::RED,         // 5
            RGBA12::LIGHT_RED,   // 6
            RGBA12::ORANGE,      // 7
            RGBA12::YELLOW,      // 8
            RGBA12::DARK_GREEN,  // 9
            RGBA12::GREEN,       // 10
            RGBA12::LIGHT_GREEN, // 11
            RGBA12::DARK_BLUE,   // 12
            RGBA12::BLUE,        // 13
            RGBA12::LIGHT_BLUE,  // 14
            RGBA12::PINK,        // 15
        ];
        self.palette_head = 16;
    }

    pub fn set_color(&mut self, id: ColorID, color: RGBA12) {
        assert!(id.0 < COLORS_PER_PALETTE as u8, "Invalid color ID");
        self.palette[id.0 as usize] = color;
    }

    pub fn palette_cycle(&mut self, color_mapping: u8, start_index: u8, end_index: u8, delta: i8) {
        let original_colors = self.color_mapping[color_mapping as usize];

        for index in start_index as isize..=end_index as isize {
            let mut new_index = index + delta as isize;
            if delta > 0 {
                if new_index > end_index as isize {
                    new_index = start_index as isize;
                }
            } else {
                if new_index < start_index as isize {
                    new_index = end_index as isize;
                }
            }
            let color = &mut self.color_mapping[color_mapping as usize][index as usize];
            *color = original_colors[new_index as usize];
        }
    }

    pub fn tile_count(&self) -> usize {
        self.tile_head as usize
    }

    pub fn color_count(&self) -> u8 {
        self.palette_head
    }

    pub fn tile_capacity(&self) -> usize {
        TILES
    }

    /// Restore tile counter to a previous state (for checkpoint/restore)
    /// Warning: Caller must ensure this is a valid previous state!
    pub fn restore_tile_count(&mut self, count: u8) {
        assert!(count as usize <= TILES, "Invalid tile count");
        self.tile_head = count;
    }

    /// Restore palette counters to previous state (for checkpoint/restore)
    /// Warning: Caller must ensure these are valid previous states!
    pub fn restore_palette_state(&mut self, color_count: u8) {
        assert!(color_count <= COLORS_PER_PALETTE as u8, "Invalid color count");
        self.palette_head = color_count;
    }

    /// Adds a single tile, returns a TileID
    pub fn add_tile(&mut self, tile: &Tile<2>) -> TileID {
        assert!((self.tile_head as usize) < TILES, err!("Tileset capacity reached"));
        let result = TileID(self.tile_head);
        // Copy tile data to bank
        let dest_index = self.tile_head as usize;
        self.tiles[dest_index] = tile.clone();
        self.tile_head += 1;
        result
    }

    /// Get a specific tile within a tileset
    pub fn get_tile(&self, index: u8) -> Option<&Tile<2>> {
        if index < self.tile_head { Some(&self.tiles[index as usize]) } else { None }
    }
}
