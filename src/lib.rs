// #![no_std]
use core::array::from_fn;

mod sprite_grid;
pub use sprite_grid::*; // testing only, TODO: remove

mod bg;
pub use bg::*;

mod color;
pub use color::*;

mod data;
pub use data::*;

mod error;

mod iter;
use iter::*;

mod pixels;
pub use pixels::*;

mod tile;
pub use tile::*;

mod scanline;
pub(crate) use scanline::*;

mod int;
pub(crate) use int::*;

// All tile dimensions must be a multiple of TILE_SIZE.
pub const TILE_SIZE: u16 = 8;
pub const MIN_TILE_SIZE: u16 = 8;
pub const MAX_TILE_SIZE: u16 = 32;
pub const SINGLE_TILE_LEN: usize = TILE_SIZE as usize * TILE_SIZE as usize; // in bytes

// Colors and bit depth
pub const PALETTE_COUNT: u8 = 16;
pub const COLORS_PER_TILE: u8 = 1 << BITS_PER_PIXEL;
pub const COLORS_PER_PALETTE: u8 = 16;

// FG Draw buffer width, in pixels. Effectively limits the maximum screen resolution
// Update TODO: With new iterator approach this could be a field!
pub const FG_WIDTH: u16 = 256;
pub const FG_HEIGHT: u16 = 192;
const LINES: usize = FG_HEIGHT as usize; // This is annoying but compiler requires it.

// BG scrollable size in pixels, also affects bg tile wrap around behavior
// Must be multiple of TILE_SIZE!
pub const BG_WIDTH: u16 = 512;
pub const BG_HEIGHT: u16 = 512;

pub const BG_COLUMNS: u16 = BG_WIDTH / TILE_SIZE;
pub const BG_ROWS: u16 = BG_HEIGHT / TILE_SIZE;

// Maximum sprite storage size in bytes (16 Kb, 256x256 pixels at 4 bpp).
const TILE_MEM_LEN: usize = 8182;

/// Main drawing context that manages the screen, tiles, and palette.
#[derive(Debug, Clone)]
pub struct VideoChip {
    /// Fixed BG Tilemap
    pub bg_map: BGMap,
    /// The color rendered if resulting pixel is transparent
    pub bg_color: ColorID,
    /// Global RGB Color Palettes (16 colors).
    pub fg_palette: [ColorRGB; COLORS_PER_PALETTE as usize],
    pub bg_palette: [ColorRGB; COLORS_PER_PALETTE as usize],
    /// Local Palettes, 16 with 4 ColorIDs each. Each ID referes to a color in the global palette.
    pub local_palettes: [[ColorID; COLORS_PER_TILE as usize]; PALETTE_COUNT as usize],
    /// Maps all i16 values into the u8 range
    pub wrap_sprites: bool,
    /// Repeats the BG Map outside its borders
    pub wrap_bg: bool,

    pub scroll_x: i16,
    pub scroll_y: i16,

    // ---------------------- Main Data ----------------------
    // The width of the visible pixels. Does not include the scrolling buffer.
    width: u16,
    // The height of the visible pixels. Does not include the scrolling buffer.
    height: u16,
    // Pixel buffers
    sprite_grid: SpriteGrid<LINES>,
    // Pixel data for all tiles, stored as palette indices.
    // Max 64Kb or 256 tiles, whichever runs out first!
    tile_pixels: [PixelCluster; TILE_MEM_LEN],
    // Array of sprite definitions. Max 256 (1 byte indices)
    tiles: [TileEntry; 256],

    // ---------------------- Bookkeeping ----------------------
    // Next available sprite ID.
    tile_id_head: u8,
    // Next available pixel position in the sprite buffer.
    tile_pixel_head: u16,
    // Next available palette.
    palette_head: u8,
    // view rect cache
    view_left: u16,
    view_top: u16,
    view_width: u16,
    view_height: u16,
}

impl VideoChip {
    /// Creates a new drawing context with default settings.
    pub fn new(width: u16, height: u16) -> Self {
        assert!(
            width > 7 && width < 257,
            err!("Screen width range is 8 to 256")
        );
        assert!(
            height > 7 && height < 257,
            err!("Screen height range is 8 to 256")
        );
        assert!(
            BG_WIDTH % TILE_SIZE == 0,
            err!("BG_WIDTH must be a multiple of TILE_SIZE")
        );
        assert!(
            BG_HEIGHT % TILE_SIZE == 0,
            err!("BG_HEIGHT must be a multiple of TILE_SIZE")
        );

        let mut result = Self {
            // fg_pixels: [0; FG_LEN],
            sprite_grid: SpriteGrid::new(width, height),
            bg_map: BGMap::new(),
            tile_pixels: [PixelCluster::default(); TILE_MEM_LEN],
            bg_color: GRAY,
            wrap_sprites: true,
            wrap_bg: true,
            tiles: [TileEntry::default(); 256],
            fg_palette: [ColorRGB::default(); COLORS_PER_PALETTE as usize],
            bg_palette: [ColorRGB::default(); COLORS_PER_PALETTE as usize],
            local_palettes: [[ColorID(0); COLORS_PER_TILE as usize]; PALETTE_COUNT as usize],
            width,
            height,
            tile_id_head: 0,
            tile_pixel_head: 0,
            palette_head: 0,
            view_left: 0,
            view_top: 0,
            view_width: width,
            view_height: height,
            scroll_x: 0,
            scroll_y: 0,
        };
        result.reset_all();

        println!(
            "Total Size of VideoChip:\t{:.1} Kb",
            size_of::<VideoChip>() as f32 / 1024.0
        );
        println!(
            "   Tile Memory:\t\t\t{:.1} Kb",
            (result.tile_pixels.len() * size_of::<PixelCluster>()) as f32 / 1024.0
        );
        println!(
            "   Sprite Grid:\t\t\t{:.1} Kb",
            size_of::<SpriteGrid<LINES>>() as f32 / 1024.0
        );
        println!(
            "   Tile entries:\t\t{:.1} Kb",
            (result.tiles.len() * size_of::<TileEntry>()) as f32 / 1024.0
        );
        println!(
            "   BG Map:\t\t\t{:.1} Kb",
            size_of::<BGMap>() as f32 / 1024.0
        );
        println!(
            "Size of PixelIter:\t{:.1} Kb",
            size_of::<PixelIter>() as f32 / 1024.0
        );

        result
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn tile(&self, tile_id: TileID) -> TileEntry {
        self.tiles[tile_id.0 as usize]
    }

    /// Does not affect BG or Sprites calculation, but "masks" PixelIter with BG Color!
    pub fn set_viewport(&mut self, left: u16, top: u16, w: u16, h: u16) {
        self.view_left = left;
        self.view_top = top;
        self.view_width = w;
        self.view_height = h;
    }

    pub fn reset_all(&mut self) {
        self.bg_color = GRAY;
        self.wrap_sprites = true;
        self.reset_scroll();
        self.reset_tiles();
        self.reset_palettes();
        self.reset_bgmap();
        self.reset_viewport();
    }

    pub fn reset_tiles(&mut self) {
        self.tile_id_head = 0;
    }

    pub fn reset_palettes(&mut self) {
        self.fg_palette = from_fn(|i| {
            if i < PALETTE_DEFAULT.len() {
                PALETTE_DEFAULT[i]
            } else {
                ColorRGB::default()
            }
        });
        self.bg_palette = from_fn(|i| {
            if i < PALETTE_DEFAULT.len() {
                PALETTE_DEFAULT[i]
            } else {
                ColorRGB::default()
            }
        });
        self.local_palettes = from_fn(|_| from_fn(|i| ColorID(i as u8)));
        self.palette_head = 0;
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_x = 0;
        self.scroll_y = 0;
    }

    pub fn reset_bgmap(&mut self) {
        self.bg_map = BGMap::new();
    }

    pub fn reset_viewport(&mut self) {
        self.view_left = 0;
        self.view_top = 0;
        self.view_width = self.width;
        self.view_height = self.height;
    }

    pub fn set_palette(&mut self, index: PaletteID, colors: [ColorID; COLORS_PER_TILE as usize]) {
        debug_assert!(
            index.0 < PALETTE_COUNT,
            err!("Invalid local palette index, must be less than PALETTE_COUNT")
        );
        self.local_palettes[index.0 as usize] = colors;
    }

    pub fn push_palette(&mut self, colors: [ColorID; COLORS_PER_TILE as usize]) -> PaletteID {
        assert!(self.palette_head < 16, err!("PALETTE_COUNT exceeded"));
        let result = self.palette_head;
        self.local_palettes[self.palette_head as usize] = colors;
        self.palette_head += 1;
        PaletteID(result)
    }

    /// Creates a new sprite from the provided pixel data.
    /// Returns a unique identifier for the newly created sprite.
    /// Panics if there's not enough space for the new sprite or
    /// if the length of data doesn't match w * h.
    ///
    /// # Parameters
    /// - `w`: Width of the sprite
    /// - `h`: Height of the sprite
    /// - `data`: Array of bytes representing the sprite pixels
    /// (each byte is a local palette index, must be in range 0-3)
    pub fn new_tile(&mut self, w: u16, h: u16, data: &[u8]) -> TileID {
        let tile_id = self.tile_id_head;
        let pixel_start = self.tile_pixel_head as usize;
        let len = (w as usize * h as usize) / 4; // Ceiling division for required bytes

        // Check if we have enough space
        if self.tile_id_head == 255 || pixel_start + len > TILE_MEM_LEN {
            panic!(err!("Not enough space for new tile"))
        }

        // Assert tile dimensions
        assert!(
            w % TILE_SIZE == 0 && h % TILE_SIZE == 0,
            err!("Tile dimensions are not multiple of TILE_SIZE")
        );

        assert!(
            w >= MIN_TILE_SIZE && w <= MAX_TILE_SIZE && h >= MIN_TILE_SIZE && h <= MAX_TILE_SIZE,
            err!("Tile dimensions outside MIN_TILE_SIZE and MAX_TILE_SIZE")
        );

        // Assert that data length is correct
        assert!(
            data.len() == w as usize * h as usize,
            err!("Tile data length does not match w * h")
        );

        // Pack 4 pixels (2 bits each) into each byte using set_subpixel
        for i in 0..data.len() {
            // Clamp color to maximum allowed
            let value = data[i].clamp(0, COLORS_PER_TILE as u8);
            // Acquire indices
            let byte_index = i / PIXELS_PER_CLUSTER as usize;
            let subpixel_index = i % PIXELS_PER_CLUSTER as usize;
            // Set pixel data
            let cluster = &mut self.tile_pixels[pixel_start + byte_index];
            cluster.set_subpixel(SubPixel(value), subpixel_index);
        }

        let cluster_index = self.tile_pixel_head;
        self.tiles[tile_id as usize] = TileEntry {
            w,
            h,
            cluster_index,
        };

        self.tile_id_head += 1;
        self.tile_pixel_head += len as u16;
        TileID(tile_id)
    }

    pub fn draw_sprite(&mut self, data: DrawBundle) {
        if data.id.0 >= self.tile_id_head {
            return;
        }

        // Handle wrapping if enabled
        let mut x = data.x - self.scroll_x;
        let mut y = data.y - self.scroll_y;

        if self.wrap_sprites {
            x = (data.x - self.scroll_x).rem_euclid(FG_WIDTH as i16);
            y = (data.y - self.scroll_y).rem_euclid(FG_HEIGHT as i16);
        }

        // Insert the sprite into the grid
        let tile = self.tiles[data.id.0 as usize];
        let tile_len = tile.w as u16 * tile.h as u16;
        let range = tile.cluster_index as usize..(tile.cluster_index + tile_len) as usize;
        let clusters = &self.tile_pixels[range];
        self.sprite_grid.insert(
            clusters,
            TileBundle {
                flags: data.flags,
                tile,
                x,
                y,
            },
        );
    }

    pub fn color_cycle(&mut self, palette: PaletteID, color: u8, min: u8, max: u8) {
        let color_cycle = &mut self.local_palettes[palette.id()][color as usize].0;
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

    pub fn start_frame(&mut self) {
        // Clear the sprite grid for the new frame
        self.sprite_grid.clear();
    }

    /// Returns an iterator over the visible screen pixels, yielding RGB colors for each pixel.
    pub fn iter_pixels(&self) -> PixelIter {
        PixelIter::new(self)
    }
}
