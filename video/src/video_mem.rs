use crate::*;
use core::array::from_fn;

#[derive(Debug, Clone)]
pub struct VideoMemory<const TILES: usize> {
    pub tiles: [Tile<2>; TILES],
    pub palette: [RGBA12; COLORS_PER_PALETTE as usize],
    /// Local Palettes, 16 with 4 ColorIDs each. Each ID referes to a color in the main palette.
    pub sub_palettes: [[ColorID; COLORS_PER_TILE as usize]; SUBPALETTE_COUNT as usize],
    // Everything that needs to be counted
    tile_head: u8,
    palette_head: u8,
    sub_palette_head: u8,
}

impl<const TILES: usize> VideoMemory<TILES> {
    pub fn new() -> Self {
        Self {
            tiles: from_fn(|_| Tile::default()),
            // bg: BGMap::new(32, 32),
            palette: PALETTE_DEFAULT,
            sub_palettes: from_fn(|_| from_fn(|i| ColorID(i as u8))),
            tile_head: 0,
            palette_head: 0,
            sub_palette_head: 0,
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
        self.sub_palettes = from_fn(|_| from_fn(|i| ColorID(i as u8)));
        self.sub_palette_head = 0;
        self.palette_head = 0;
    }

    pub fn push_color(&mut self, color: RGBA12) -> ColorID {
        assert!(
            self.palette_head < COLORS_PER_PALETTE as u8,
            "Palette capacity reached"
        );
        let id = ColorID(self.palette_head);
        self.palette[self.palette_head as usize] = color;
        self.palette_head += 1;
        id
    }

    pub fn set_color(&mut self, id: ColorID, color: RGBA12) {
        assert!(
            id.0 < COLORS_PER_PALETTE as u8,
            "Invalid color ID"
        );
        self.palette[id.0 as usize] = color;
    }

    pub fn set_subpalette(
        &mut self,
        index: PaletteID,
        colors: [ColorID; COLORS_PER_TILE as usize],
    ) {
        assert!(
            index.0 < SUBPALETTE_COUNT,
            err!("Invalid local palette index, must be less than PALETTE_COUNT")
        );
        self.sub_palettes[index.0 as usize] = colors;
    }

    pub fn push_subpalette(&mut self, colors: [ColorID; COLORS_PER_TILE as usize]) -> PaletteID {
        assert!(
            self.sub_palette_head < SUBPALETTE_COUNT,
            err!("SUBPALETTE_COUNT exceeded")
        );
        let result = self.sub_palette_head;
        self.sub_palettes[self.sub_palette_head as usize] = colors;
        self.sub_palette_head += 1;
        PaletteID(result)
    }

    /// Increments or decrements an index in a local palette so that its value
    /// cycles between "min" and "max", which represent colors in the Main FG and BG palettes.
    pub fn color_cycle(&mut self, palette: PaletteID, color: u8, min: u8, max: u8) {
        let color_cycle = &mut self.sub_palettes[palette.id()][color as usize].0;
        if max > min {
            *color_cycle += 1;
            if *color_cycle > max {
                *color_cycle = min
            }
        } else {
            *color_cycle -= 1;
            if *color_cycle < min {
                *color_cycle = max
            }
        }
    }

    pub fn tile_count(&self) -> usize {
        self.tile_head as usize
    }

    pub fn color_count(&self) -> u8 {
        self.palette_head
    }

    pub fn sub_palette_count(&self) -> u8 {
        self.sub_palette_head
    }

    pub fn tile_capacity(&self) -> usize {
        TILES
    }

    /// Adds a single tile, returns a TileID
    pub fn add_tile(&mut self, tile: &Tile<2>) -> TileID {
        assert!(
            (self.tile_head as usize) < TILES,
            err!("Tileset capacity reached")
        );
        let result = TileID(self.tile_head);
        // Copy tile data to bank
        let dest_index = self.tile_head as usize;
        self.tiles[dest_index] = tile.clone();
        self.tile_head += 1;
        result
    }

    /// Get a specific tile within a tileset
    pub fn get_tile(&self, index: u8) -> Option<&Tile<2>> {
        if index < self.tile_head {
            Some(&self.tiles[index as usize])
        } else {
            None
        }
    }
}
