use core::array;

use crate::*;
use slotmap::SlotMap;

/// Loads and stores fixed size tiles organized into tilesets that can be added and removed individually.
pub struct Atlas<
    const PIXEL_COUNT:usize,
    const TILE_COUNT:usize,
    const ANIM_COUNT:usize,
    const FONT_COUNT:usize,
    const TILEMAP_COUNT:usize,
    const PALETTE_COUNT:usize,
    const COLORS_PER_PALETTE:usize
> {
    pub(crate) rects:[Rect<u8>; TILE_COUNT],
    pub(crate) fonts: Pool<Font, FONT_COUNT>,
    pub(crate) anims: Pool<Anim, ANIM_COUNT>,
    pub(crate) tilemaps: Pool<Tilemap, TILEMAP_COUNT>,
    pub(crate) palettes: Pool<Palette<COLORS_PER_PALETTE>, PALETTE_COUNT>,    //TODO: Find all instances of Palette, remove hard coded number of colors
    pub(crate) tilesets: SlotMap<TilesetID, Tileset>,
    pixels:[u8; PIXEL_COUNT],
    // palette: [Color; 256],

    next_tileset:u16,
    next_free_tile:u16,
    width: u16,
    height:u16,
    tile_width:u8,
    tile_height:u8,
}


impl<
    const PIXEL_COUNT:usize,
    const TILE_COUNT:usize,
    const ANIM_COUNT:usize,
    const FONT_COUNT:usize,
    const TILEMAP_COUNT:usize,
    const PALETTE_COUNT:usize,
    const COLORS_PER_PALETTE:usize
> Atlas<PIXEL_COUNT, TILE_COUNT, ANIM_COUNT, FONT_COUNT, TILEMAP_COUNT, PALETTE_COUNT, COLORS_PER_PALETTE>
{
    pub(crate) fn new(width:u16, height:u16, tile_width:u8, tile_height:u8) -> Self {
        // println!("Atlas: Creating new Atlas with {} tiles.", MAX_TILES);
        assert!(PIXEL_COUNT==width as usize * height as usize , "Atlas: Error, width x height must equal PIXEL_COUNT");
        assert!(TILE_COUNT==(width as usize /tile_width as usize)*(height as usize /tile_height as usize), "Atlas: Invalid tile count.");
        Atlas {
            pixels: [0; PIXEL_COUNT],
            rects: array::from_fn( |i| {
                // generates all tiles
                let tile_x = i * tile_width as usize;
                let x = (tile_x % width as usize ) as u8;
                let y = ((tile_x / width as usize ) * tile_height as usize) as u8;
                Rect{ x ,y , w:tile_width, h:tile_height }
            }),
            fonts: Default::default(),
            anims: Default::default(),
            tilemaps: Default::default(),
            tilesets: Default::default(),
            palettes: Default::default(),
            next_tileset: 0,
            next_free_tile: 0,
            width,
            height,
            tile_width,
            tile_height
        }
    }


    pub fn width(&self) -> usize { self.width as usize  }


    pub fn height(&self) -> usize { self.height as usize }


    pub fn tile_width(&self) -> u8 { self.tile_width }


    pub fn tile_height(&self) -> u8 { self.tile_height }


    // pub fn get_color(&self, index:u8) -> Color {
    //     // TODO: Adjust for non hard coded palette size
    //     match index {
    //         COLOR_TRANSPARENCY => Color{r:0,g:255,b:0,a:255},
    //         COLOR_ENTITY_RECT => Color{r:0,g:255,b:255,a:255},
    //         COLOR_COLLIDER => Color{r:255,g:128,b:128,a:255},
    //         _ => {
    //             let color_idx = (index % 16) as usize; 
    //             let palettte_idx = (index / 16) as usize;
    //             *self.palettes.data[palettte_idx].colors.get(color_idx).unwrap()
    //         }
    //     }
    // }

    // pub fn palette(&self) -> &[Color; 256] {
    //     &self.palette
    //     // let mut palette_idx:usize = 0;
    //     // let mut color_idx:usize = 0;
    //     // core::array::from_fn(|_| {
    //     //     let mut result = self.palettes.data[palette_idx].colors[color_idx];
    //     //     color_idx += 1;
    //     //     if color_idx == self.palettes.data[palette_idx].len() {
    //     //         color_idx = 0;
    //     //         palette_idx += 1;
    //     //         if palette_idx == self.palettes.len() {
    //     //             result = Color::default();
    //     //         }
    //     //     }
    //     //     result
    //     // })
    // }


    pub fn insert_tileset( &mut self, data:&[u8] ) -> TilesetID {
        if data[0 .. TILESET_HEADER_TEXT.len()] != *TILESET_HEADER_TEXT.as_bytes() { panic!("Atlas error: Invalid .tiles file") }
        
        let mut offset = TILEMAP_HEADER_TEXT.len();
        let mut cursor = || -> usize {
            let result = offset;
            offset += 1;
            result
        };

        let tile_width = data[cursor()];
        let tile_height = data[cursor()];
        let pixel_count = u16::from_ne_bytes([data[cursor()], data[cursor()]]);
        let font_count = data[cursor()];
        let anim_count = data[cursor()];
        let tilemap_count = data[cursor()];
        let palette_id = data[cursor()];
        let palette_len = data[cursor()] as usize;

        // Wrap up header, error checking
        let tile_len = (self.tile_width * self.tile_height) as u16;
        let tile_count = pixel_count / tile_len;
        let tile_length = tile_width as u16 * tile_height as u16;

        if (tile_width != self.tile_width) || (tile_height != self.tile_height) {
            panic!("Atlas error: invalid tileset dimensions. Expected {}x{} tiles, found {}x{}",
            self.tile_width, self.tile_height, tile_width, tile_height)
        }

        if tile_count as usize * tile_length as usize != pixel_count as usize {
            panic!("Atlas error: invalid tileset dimensions. Expected {} pixels.", pixel_count)
        }
        
        // Insert new tileset
        #[cfg(std)]{ println!(
            "\nLoading tileset with {} pixels, {} colors, {} fonts, {} anims, and {} maps",
            pixel_count, palette_len, font_count, anim_count, tilemap_count
        )};
        let start_index = self.next_free_tile; 
        let len = pixel_count / tile_len; 
        #[cfg(std)]{ println!("    start_index:{}, len:{} , palette_id:{}", start_index, len, palette_id); }
        let result = self.tilesets.insert_with_key(|key| {
            Tileset {
                unique_id:key,
                start_index,
                len,
                palette_id
            }
        });

        // Load palettte. Will overwrite the same palette sometimes, since tilesets may use the same palette
        let mut palette = Palette::new(palette_id);
        #[cfg(std)]{ println!("    Loading palette {} with {} colors", palette_id, palette_len); }
        // let current = offset.clone();
        // println!("    Cursor: {}", current);
        for _i in 0 .. palette_len {
            let r = data[cursor()];
            let g = data[cursor()];
            let b = data[cursor()];
            let a = data[cursor()];
            palette.push(Color{r,g,b,a});
            #[cfg(std)]{ println!("        {:02}: {:?}", _i, Color{r,g,b,a}); }
        }
        
        self.palettes.insert(palette_id as usize, palette);

        // Load pixels from linear format into tile-formatted.
        #[cfg(std)]{
            print!("    Loading {} pixels... ", pixel_count);
            let mut pix_count:usize = 0;
        }
        let cols = self.width as usize  / self.tile_width as usize;
        for tile in  start_index as usize .. (start_index + len) as usize {
            for y in 0 .. tile_height as usize {
                for x in 0 ..tile_width as usize {
                    let col = tile % cols;
                    let row = tile / cols;
                    let tile_x = col * tile_width as usize;
                    let tile_y = row * tile_height as usize;
                    let dest_px = (self.width as usize  * (tile_y + y)) + (tile_x + x);
                    self.pixels[dest_px] = data[cursor()];
                    #[cfg(std)]{ pix_count += 1; }
                }
            }
        }
        #[cfg(std)]{ println!("{} pixels loaded", pix_count); }

        // Load fonts
        for _ in 0 .. font_count {
            let id = data[cursor()];
            let start = data[cursor()];
            let len = data[cursor()]; 
            // if id as usize != self.fonts.len() { panic!("Atlas Error: Font ID does not match its Pool index!")}
            let font = Font { start, len, id, tileset:result };
            #[cfg(std)]{ println!("    Loading font:{:?}", font); }
            self.fonts.insert(id as usize, font);
        }

        // Load Anims
        for _ in 0 .. anim_count {
            let id = data[cursor()];
            let group = data[cursor()];
            let fps = data[cursor()];
            let len = data[cursor()];
            #[cfg(std)]{ println!("    Loading anim {} with group {}", id, group); }
            self.anims.insert (
                id as usize,
                Anim {
                    id,
                    group,
                    fps,
                    len,
                    frames: core::array::from_fn(|_frame|{
                        #[cfg(std)]{ print!("        Loading frame {} ", _frame); }
                        Frame {
                            cols: data[cursor()],
                            rows: data[cursor()],
                            tiles: core::array::from_fn(|_tile|{
                                let index = data[cursor()];
                                #[cfg(std)]{
                                    print!("    tile {}:{},", tile, index);
                                    if _tile == ANIM_TILES_PER_FRAME-1 { println!("") }
                                }
                                Tile {
                                    index,
                                    flags: data[cursor()]
                                }
                            })
                        }
                    }),
                    tileset: result,
                }
            );
        }

        // Load Tilemaps
        for _ in 0 .. tilemap_count {
            let id = data[cursor()];
            let cols = u16::from_ne_bytes([data[cursor()], data[cursor()]]);
            let rows = u16::from_ne_bytes([data[cursor()], data[cursor()]]);
            #[cfg(std)]{ println!("    Loading map {} with {}x{} tiles", id, cols, rows); }
            self.tilemaps.insert(
                id as usize,
                Tilemap{
                    id,
                    tileset: result,
                    cols,
                    rows,
                    bg_buffers: Default::default(),
                    tiles: core::array::from_fn(|_|{
                        Tile{
                            index: data[cursor()],
                            flags: data[cursor()],
                        }
                    }
                )
            });

        }
        
        
        // Finish tileset insertion
        if offset != data.len() {
            panic!("Atlas error: expected file length is {}, found {}", data.len(), offset)
        }

        self.next_tileset += 1;
        self.next_free_tile += len;
        result
    }


    pub fn remove_tileset(&mut self, id:TilesetID) {
        self.tilesets.remove(id);
    }


    pub fn get_tileset(&self, id:TilesetID) -> &Tileset {
        &self.tilesets[id]
    }


    pub fn get_tile_and_palette(&self, index:u8, tileset_id:TilesetID) -> (TileID, &Palette<COLORS_PER_PALETTE>) {
        let tileset = &self.tilesets[tileset_id];
        let palette = self.palettes.get(tileset.palette_id as usize).unwrap();
        // let result = TileID(tileset.start_index + index as u16);
        // result
        (TileID(tileset.start_index + index as u16), palette)
    }


    pub fn get_rect(&self, index:usize) -> Rect<u8> {
        self.rects[index]
    }


    pub fn get_pixel(&self, x:usize, y:usize) -> u8 {
        let index = (y * self.width as usize) + x;
        self.pixels[index]
    }


    pub fn get_anims(&self) -> &Pool<Anim, ANIM_COUNT> { &self.anims }

    // Slower methods, since they calculate the absolute tile coordinates on every pixel
    // #[inline]
    // pub fn get_pixel_from_quad(&self, x:usize, y:usize, tile_id:TileID) -> u8 {
    //     let (x,y) = self.get_coords_from_quad(x, y, tile_id);
    //     // Return pixel
    //     let index = (y * ATLAS_WIDTH) + x;
    //     self.pixels[index]
    // }
    // pub fn set_pixel_in_quad(&mut self, x:usize, y:usize, tile_id:TileID, color_index:u8) {
    //     let (x,y) = self.get_coords_from_quad(x, y, tile_id);
    //     // Return pixel
    //     let index = (y * ATLAS_WIDTH) + x;
    //     self.pixels[index] = color_index;
    // }
    // pub fn get_coords_from_quad(&self, x:usize, y:usize, tile_id:TileID) -> (usize, usize) {
    //     // Acquire rect
    //     let id = tile_id.get();
    //     let rect = self.tiles[id];
    //     // Wrap coordinates
    //     let x = x % usize::from(rect.w);
    //     let y = y % usize::from(rect.h);
    //     // Atlas space
    //     let x = usize::from(rect.x) + x;
    //     let y = usize::from(rect.y) + y;
    //     (x, y)
    // }

    
}