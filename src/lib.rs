#![no_std]
use core::array::from_fn;

pub mod prelude {
    pub use crate::color::*;
    pub use crate::data::*;
}

pub mod color;
pub mod data;

// Visible by default
pub use color::*;

mod error;

mod bg;
pub use bg::*;

mod iter;
pub use iter::*;

mod cluster;
pub use cluster::*;

mod tile;
pub use tile::*;

/// FG Draw buffer height.
pub const LINE_COUNT: usize = 196;

/// All tile dimensions must be a multiple of TILE_SIZE.
pub const TILE_SIZE: u8 = 8;

// Colors and bit depth
/// Number of colors per tile (2 bits per pixel)
pub const COLORS_PER_TILE: u8 = 4;

/// Number of colors per palette (applies to FG and BG palette, 32 colors total)
pub const COLORS_PER_PALETTE: u8 = 16;

/// How many "local" palettes
/// (palettes of 4 colors that map each index to the main FG and BG palettes)
pub const LOCAL_PALETTE_COUNT: u8 = 16;

/// 4 pixels per byte (4 colors per pixel)
pub const SUBPIXELS_TILE: u8 = Cluster::<2>::PIXELS_PER_BYTE as u8;

/// 2 pixels per byte (16 colors per pixel)
pub const SUBPIXELS_FRAMEBUFFER: u8 = Cluster::<4>::PIXELS_PER_BYTE as u8;

/// Number of columns in BG Map
pub const BG_COLUMNS: u8 = 64;

/// Number of rows in BG Map
pub const BG_ROWS: u8 = 64;

/// Number of columns in BG Map times tile size, in pixels.
pub const BG_WIDTH: u16 = BG_COLUMNS as u16 * TILE_SIZE as u16;

/// Number of rows in BG Map times tile size, in pixels.
pub const BG_HEIGHT: u16 = BG_ROWS as u16 * TILE_SIZE as u16;

/// Maximum sprite storage length (16 Kb with Cluster<2> used).
const TILE_MEM_LEN: usize = 8182;

/// A convenient packet of data used to draw a tile as a sprite.
#[derive(Debug, Clone, Copy)]
pub struct DrawBundle {
    pub x: i16,
    pub y: i16,
    pub id: TileID,
    pub flags: TileFlags,
}

/// Main drawing context that manages the screen, tiles, and palette.
#[derive(Debug, Clone)]
pub struct VideoChip {
    /// Fixed BG Tilemap
    pub bg_map: BGMap,
    /// The color rendered if resulting pixel is transparent
    pub bg_color: ColorID,
    /// The main FG palette with 16 colors. Used by sprites.
    pub fg_palette: [Color9Bit; COLORS_PER_PALETTE as usize],
    /// The main BG palette with 16 colors. Used by BG tiles.
    pub bg_palette: [Color9Bit; COLORS_PER_PALETTE as usize],
    /// Local Palettes, 16 with 4 ColorIDs each. Each ID referes to a color in the global palette.
    pub local_palettes: [[ColorID; COLORS_PER_TILE as usize]; LOCAL_PALETTE_COUNT as usize],
    /// Maps i16 coordinates into the u8 range, bringing sprites "outside the screen" into view.
    pub wrap_sprites: bool,
    /// Repeats the BG Map outside its borders
    pub wrap_bg: bool,
    /// Offsets the BG Map and Sprite tiles horizontally
    pub scroll_x: i16,
    /// Offsets the BG Map and Sprite tiles vertically
    pub scroll_y: i16,

    // ---------------------- Main Data ----------------------
    // TODO: Make pub(Crate) after testing
    scanlines: [[Cluster<4>; 256 / PIXELS_PER_CLUSTER as usize]; LINE_COUNT],
    crop_x: u8,
    crop_y: u8,
    max_x: u8,
    max_y: u8,
    // Pixel buffers
    // sprite_grid: SpriteGrid,
    // Pixel data for all tiles, stored as palette indices.
    // Max 64Kb or 256 tiles, whichever runs out first!
    tile_pixels: [Cluster<2>; TILE_MEM_LEN], // 2 bits per pixel
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
    view_left: u8,
    view_top: u8,
    view_right: u8,
    view_bottom: u8,
}

// TODO: change width and height into max_x and max_y

impl VideoChip {
    /// Creates a new drawing context with default settings.
    pub fn new(w: u32, h: u32) -> Self {
        assert!(w > 7 && w < 257, err!("Screen width range is 8 to 256"));
        assert!(h > 7 && h < 257, err!("Screen height range is 8 to 256"));
        assert!(LINE_COUNT < 256, err!("LINE_COUNT must be less than 256"));

        let mut result = Self {
            // fg_pixels: [0; FG_LEN],
            // sprite_grid: SpriteGrid::new(),
            bg_map: BGMap::new(),
            tile_pixels: [Cluster::default(); TILE_MEM_LEN],
            bg_color: GRAY,
            wrap_sprites: true,
            wrap_bg: true,
            tiles: [TileEntry::default(); 256],
            fg_palette: [Color9Bit::default(); COLORS_PER_PALETTE as usize],
            bg_palette: [Color9Bit::default(); COLORS_PER_PALETTE as usize],
            local_palettes: [[ColorID(0); COLORS_PER_TILE as usize]; LOCAL_PALETTE_COUNT as usize],
            scanlines: from_fn(|_| from_fn(|_| Cluster::default())),
            max_x: (w - 1) as u8,
            max_y: (h - 1) as u8,
            tile_id_head: 0,
            tile_pixel_head: 0,
            palette_head: 0,
            view_left: 0,
            view_top: 0,
            view_right: (w - 1) as u8,
            view_bottom: (h - 1) as u8,
            crop_x: 0,
            crop_y: 0,
            scroll_x: 0,
            scroll_y: 0,
        };
        result.reset_all();

        // println!(
        //     "Total Size of VideoChip:\t{:.1} Kb",
        //     size_of::<VideoChip>() as f32 / 1024.0
        // );
        // println!(
        //     "   Tile Memory:\t\t\t{:.1} Kb",
        //     (result.tile_pixels.len() * size_of::<Cluster<2>>()) as f32 / 1024.0
        // );
        // println!(
        //     "   Tile entries:\t\t{:.1} Kb",
        //     (result.tiles.len() * size_of::<TileEntry>()) as f32 / 1024.0
        // );
        // println!(
        //     "   BG Map:\t\t\t{:.1} Kb",
        //     size_of::<BGMap>() as f32 / 1024.0
        // );
        // println!(
        //     "Size of PixelIter:\t{:.1} Kb",
        //     size_of::<PixelIter>() as f32 / 1024.0
        // );

        result
    }

    pub fn max_x(&self) -> u8 {
        self.max_x
    }

    pub fn max_y(&self) -> u8 {
        self.max_y
    }

    pub fn width(&self) -> u32 {
        self.max_x as u32 + 1
    }

    pub fn height(&self) -> u32 {
        self.max_y as u32 + 1
    }

    pub fn tile_entry(&self, tile_id: TileID) -> TileEntry {
        self.tiles[tile_id.0 as usize]
    }

    pub fn crop_x(&self) -> u8 {
        self.crop_x
    }

    pub fn set_crop_x(&mut self, value: u8) {
        assert!(
            value < 255 - self.max_x + 1,
            err!("crop_x must be lower than 255 - width")
        );
        self.crop_x = value;
    }

    pub fn crop_y(&self) -> u8 {
        self.crop_y
    }

    pub fn set_crop_y(&mut self, value: u8) {
        assert!(
            value < 255 - self.max_y + 1,
            err!("crop_y must be lower than 255 - height")
        );
        self.crop_y = value;
    }

    /// Does not affect BG or Sprites calculation, but "masks" PixelIter pixels outside
    /// this rectangular area with the BG Color
    pub fn set_viewport(&mut self, left: u8, top: u8, w: u8, h: u8) {
        self.view_left = left;
        self.view_top = top;
        self.view_right = left.saturating_add(w);
        self.view_bottom = top.saturating_add(h);
    }

    /// Resets the chip to its initial state.
    pub fn reset_all(&mut self) {
        self.bg_color = GRAY;
        self.wrap_sprites = true;
        self.reset_scroll();
        self.reset_tiles();
        self.reset_palettes();
        self.reset_bgmap();
        self.reset_crop();
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
                Color9Bit::default()
            }
        });
        self.bg_palette = from_fn(|i| {
            if i < PALETTE_DEFAULT.len() {
                PALETTE_DEFAULT[i]
            } else {
                Color9Bit::default()
            }
        });
        self.local_palettes = from_fn(|_| from_fn(|i| ColorID(i as u8)));
        self.palette_head = 0;
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_x = 0;
        self.scroll_y = 0;
    }

    pub fn reset_crop(&mut self) {
        self.crop_x = 0;
        self.crop_y = 0;
    }

    pub fn reset_bgmap(&mut self) {
        self.bg_map = BGMap::new();
    }

    pub fn reset_viewport(&mut self) {
        self.view_left = 0;
        self.view_top = 0;
        self.view_right = self.max_x;
        self.view_bottom = self.max_y;
    }

    pub fn set_palette(&mut self, index: PaletteID, colors: [ColorID; COLORS_PER_TILE as usize]) {
        debug_assert!(
            index.0 < LOCAL_PALETTE_COUNT,
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

    /// Creates a new tile from the provided pixel data, and returns a unique identifier.
    /// Panics if there's not enough space for the new sprite or
    /// if the length of data doesn't match w * h.
    pub fn new_tile(&mut self, w: u8, h: u8, data: &[u8]) -> TileID {
        let tile_id = self.tile_id_head;
        let pixel_start = self.tile_pixel_head as usize;

        // Each Cluster<2> holds 8 pixels, so divide by 8 to get cluster count
        let len = (w as usize * h as usize + 7) / 8; // Ceiling division for required clusters

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
            w >= TILE_SIZE && h >= TILE_SIZE,
            err!("Tile dimensions must be TILE_SIZE or larger")
        );

        // Assert that data length is correct
        assert!(
            data.len() == w as usize * h as usize,
            err!("Tile data length does not match w * h")
        );

        // Pack 8 pixels (2 bits each) into each cluster
        for i in 0..data.len() {
            // Clamp color to maximum allowed
            let value = data[i].clamp(0, COLORS_PER_TILE as u8);

            // Acquire indices - each cluster holds 8 pixels
            let cluster_index = i / 8; // 8 pixels per cluster
            let subpixel_index = (i % 8) as u8;

            // Set pixel data
            let cluster = &mut self.tile_pixels[pixel_start + cluster_index];
            cluster.set_subpixel(value, subpixel_index);
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

    /// Draws a tile anywhere on the screen using i16 coordinates for convenience. You can
    /// also provide various tile flags, like flipping, and specify a palette id.
    pub fn draw_sprite(&mut self, data: DrawBundle) {
        if data.id.0 >= self.tile_id_head {
            return;
        }

        // Handle wrapping
        let wrapped_x: u8;
        let wrapped_y: u8;
        if self.wrap_sprites {
            wrapped_x = (data.x - self.scroll_x).rem_euclid(256) as u8;
            wrapped_y = (data.y - self.scroll_y).rem_euclid(LINE_COUNT as i16) as u8;
        } else {
            let max_x = self.scroll_x + self.max_x as i16 + self.crop_x as i16;
            if data.x < self.scroll_x || data.x > max_x {
                return;
            } else {
                wrapped_x = (data.x - self.scroll_x) as u8;
            }
            let max_y = self.scroll_y + self.max_y as i16 + self.crop_y as i16;
            if data.y < self.scroll_y || data.y > max_y {
                return;
            } else {
                wrapped_y = (data.y - self.scroll_y).clamp(0, LINE_COUNT as i16 - 1) as u8;
            }
        }

        // Get tile info
        let tile = self.tiles[data.id.0 as usize];

        // Calculate effective sprite dimensions based on rotation
        let (width, height) = if data.flags.is_rotated() {
            (tile.h, tile.w) // Swap width and height for rotated sprites
        } else {
            (tile.w, tile.h)
        };

        // Calculate sprite boundaries in screen coordinates
        let right_bound = if (wrapped_x as u16 + width as u16) > 255 {
            255
        } else {
            wrapped_x + width
        };

        let bottom_bound = if (wrapped_y as u16 + height as u16) > LINE_COUNT as u16 {
            LINE_COUNT as u8
        } else {
            wrapped_y + height
        };

        // Process each visible scanline
        for screen_y in wrapped_y..bottom_bound {
            let local_y = screen_y - wrapped_y;

            for screen_x in wrapped_x..right_bound {
                let local_x = screen_x - wrapped_x;

                let (tx, ty) =
                    Self::transform_tile_coords(local_x, local_y, width, height, data.flags);

                // Calculate pixel position within the tile
                let pixel_index = ty as usize * tile.w as usize + tx as usize;

                // Calculate which cluster contains this pixel
                let cluster_index = pixel_index / PIXELS_PER_CLUSTER as usize;

                // Calculate the subpixel index within the cluster
                let subpixel_index = (pixel_index % PIXELS_PER_CLUSTER as usize) as u8;

                // Get the pixel color from the tile
                let cluster = self.tile_pixels[(tile.cluster_index as usize) + cluster_index];
                let color_index = cluster.get_subpixel(subpixel_index);

                // If not transparent, draw it to the scanline
                if color_index > 0 {
                    // Translate to global palette color
                    let palette_index = data.flags.palette().0 as usize;
                    let color_id = self.local_palettes[palette_index][color_index as usize];

                    // Calculate scanline cluster index and position
                    let scanline_cluster = screen_x as usize / PIXELS_PER_CLUSTER as usize;
                    let scanline_subpixel = (screen_x % PIXELS_PER_CLUSTER) as u8;

                    // Set the pixel in the scanline
                    self.scanlines[screen_y as usize][scanline_cluster]
                        .set_subpixel(color_id.0, scanline_subpixel);
                }
            }
        }
    }

    #[inline(always)]
    pub(crate) fn transform_tile_coords(x: u8, y: u8, w: u8, h: u8, flags: TileFlags) -> (u8, u8) {
        // Handle both rotation and flipping
        if flags.is_rotated() {
            // For 90° clockwise rotation, swap x and y and flip the new x axis
            let rotated_x = h - 1 - y;
            let rotated_y = x;

            // Apply additional flipping if needed
            if flags.is_flipped_x() {
                // Flipping X after 90° rotation is equivalent to flipping the new Y
                (rotated_x, w - 1 - rotated_y)
            } else if flags.is_flipped_y() {
                // Flipping Y after 90° rotation is equivalent to flipping the new X
                (h - 1 - rotated_x, rotated_y)
            } else {
                (rotated_x, rotated_y)
            }
        } else {
            // Handle just flipping without rotation
            let flipped_x = if flags.is_flipped_x() { w - 1 - x } else { x };
            let flipped_y = if flags.is_flipped_y() { h - 1 - y } else { y };
            (flipped_x, flipped_y)
        }
    }

    /// Increments or decrements an index in a local palette so that its value
    /// cycles between "min" and "max", which represent colors in the Main FG and BG palettes.
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
        for line in &mut self.scanlines {
            *line = from_fn(|_| Cluster::default());
        }
    }

    /// Returns an iterator over the visible screen pixels, yielding RGB colors for each pixel.
    pub fn iter_pixels(&self) -> PixelIter {
        PixelIter::new(self)
    }
}
