use core::array;

use crate::{*, common::tile};
// use slotmap::SlotMap;



/// Loads and stores fixed size tiles organized into tilesets that can be added and removed individually.
pub struct Atlas<S:Specs>
where
    [(); S::ATLAS_WIDTH * S::ATLAS_HEIGHT]: Sized,
    [(); S::ATLAS_TILE_COUNT]: Sized,
    [(); S::ANIM_COUNT]: Sized,
    [(); S::FONT_COUNT]: Sized,
    [(); S::TILEMAP_COUNT]: Sized,
    [(); S::TILESET_COUNT]: Sized,
    [(); S::PALETTE_COUNT]: Sized,
    [(); S::COLORS_PER_PALETTE]: Sized,
{
    pub(crate) rects:       [Rect<u8>;                                  S::ATLAS_TILE_COUNT],
    pub(crate) fonts:       [Option<Font>;                              S::FONT_COUNT],
    pub(crate) anims:       [Option<Anim>;                              S::ANIM_COUNT],
    pub(crate) tilemaps:    [Option<Tilemap>;                           S::TILEMAP_COUNT],
    pub(crate) palettes:    [Option<Palette<S>>;                        S::PALETTE_COUNT],
    pub(crate) tilesets:    [Option<Tileset>;                           S::TILESET_COUNT],

    pixels:[u8; S::ATLAS_WIDTH * S::ATLAS_HEIGHT],
    next_tileset:u16,
    next_free_tile:u16,
}


impl<S:Specs> Atlas<S>
where
    [(); S::ATLAS_TILE_COUNT]: Sized,
    [(); S::ANIM_COUNT]: Sized,
    [(); S::FONT_COUNT]: Sized,
    [(); S::TILEMAP_COUNT]: Sized,
    [(); S::TILESET_COUNT]: Sized,
    [(); S::PALETTE_COUNT]: Sized,
    [(); S::COLORS_PER_PALETTE]: Sized,
    [(); S::RENDER_WIDTH * S::RENDER_HEIGHT]: Sized,
    [(); S::ATLAS_WIDTH * S::ATLAS_HEIGHT]: Sized,
{
    pub(crate) fn new() -> Self {
        // println!("Atlas: Creating new Atlas with {} tiles.", MAX_TILES);
        let tile_count = (S::ATLAS_WIDTH * S::ATLAS_HEIGHT) / (S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize);
        assert!(S::ATLAS_TILE_COUNT == tile_count, "Atlas specs error: invalid tile count {}", S::ATLAS_TILE_COUNT);
        Atlas {
            pixels: [0; S::ATLAS_WIDTH * S::ATLAS_HEIGHT],
            rects: array::from_fn( |i| {
                // generates all tiles
                let tile_x = i * S::TILE_WIDTH as usize;
                let x = (tile_x % S::ATLAS_WIDTH) as u8;
                let y = ((tile_x / S::ATLAS_WIDTH) * S::TILE_HEIGHT as usize) as u8;
                Rect{
                    x,
                    y,
                    w:u8::try_from(S::TILE_WIDTH).unwrap(),
                    h:u8::try_from(S::TILE_HEIGHT).unwrap()
                }
            }),
            fonts: core::array::from_fn(|_| None ),
            anims: core::array::from_fn(|_| None ),
            tilemaps: core::array::from_fn(|_| None ),
            palettes: core::array::from_fn(|_| None ),
            tilesets: core::array::from_fn(|_| None ),
            next_tileset: 0,
            next_free_tile: 0,
        }
    }


    pub fn width(&self) -> usize { S::ATLAS_WIDTH }


    pub fn height(&self) -> usize { S::ATLAS_HEIGHT }


    pub fn tile_width(&self) -> u8 { u8::try_from(S::TILE_WIDTH).unwrap() }


    pub fn tile_height(&self) -> u8 { u8::try_from(S::TILE_HEIGHT).unwrap() }


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


    // pub fn insert_tileset( &mut self, data:&[u8] ) -> TilesetID {
    //     if data[0 .. TILESET_HEADER_TEXT.len()] != *TILESET_HEADER_TEXT.as_bytes() { panic!("Atlas error: Invalid .tiles file") }
        
    //     let mut offset = TILEMAP_HEADER_TEXT.len();
    //     let mut cursor = || -> usize {
    //         let result = offset;
    //         offset += 1;
    //         result
    //     };

    //     let tile_width = data[cursor()];
    //     let tile_height = data[cursor()];
    //     let pixel_count = u16::from_ne_bytes([data[cursor()], data[cursor()]]);
    //     let font_count = data[cursor()];
    //     let anim_count = data[cursor()];
    //     let tilemap_count = data[cursor()];
    //     let palette_id = data[cursor()];
    //     let palette_len = data[cursor()] as usize;

    //     // Wrap up header, error checking
    //     let tile_len = (self.tile_width * self.tile_height) as u16;
    //     let tile_count = pixel_count / tile_len;
    //     let tile_length = tile_width as u16 * tile_height as u16;

    //     if (tile_width != self.tile_width) || (tile_height != self.tile_height) {
    //         panic!("Atlas error: invalid tileset dimensions. Expected {}x{} tiles, found {}x{}",
    //         self.tile_width, self.tile_height, tile_width, tile_height)
    //     }

    //     if tile_count as usize * tile_length as usize != pixel_count as usize {
    //         panic!("Atlas error: invalid tileset dimensions. Expected {} pixels.", pixel_count)
    //     }
        
    //     // Insert new tileset
    //     #[cfg(std)]{ println!(
    //         "\nLoading tileset with {} pixels, {} colors, {} fonts, {} anims, and {} maps",
    //         pixel_count, palette_len, font_count, anim_count, tilemap_count
    //     )};
    //     let start_index = self.next_free_tile; 
    //     let len = pixel_count / tile_len; 
    //     #[cfg(std)]{ println!("    start_index:{}, len:{} , palette_id:{}", start_index, len, palette_id); }
    //     let result = self.tilesets.insert_with_key(|key| {
    //         Tileset {
    //             unique_id:key,
    //             start_index,
    //             len,
    //             palette_id
    //         }
    //     });

    //     // Load palettte. Will overwrite the same palette sometimes, since tilesets may use the same palette
    //     let mut palette = Palette::new(palette_id);
    //     #[cfg(std)]{ println!("    Loading palette {} with {} colors", palette_id, palette_len); }
    //     // let current = offset.clone();
    //     // println!("    Cursor: {}", current);
    //     // let mut color_index = 0;
    //     for _i in 0 .. palette_len {
    //         let r = data[cursor()];
    //         let g = data[cursor()];
    //         let b = data[cursor()];
    //         let a = data[cursor()];
    //         palette.push(Color{r,g,b,a});
    //         // palette.colors[color_index] = Color{r,g,b,a};
    //         // color_index += 1;
    //         #[cfg(std)]{ println!("        {:02}: {:?}", _i, Color{r,g,b,a}); }
    //     }
        
    //     self.palettes[palette_id as usize] = Some(palette);

    //     // Load pixels from linear format into tile-formatted.
    //     #[cfg(std)]{
    //         print!("    Loading {} pixels... ", pixel_count);
    //         let mut pix_count:usize = 0;
    //     }
    //     let cols = S::ATLAS_WIDTH  / S::TILE_WIDTH;
    //     for tile in  start_index as usize .. (start_index + len) as usize {
    //         for y in 0 .. tile_height as usize {
    //             for x in 0 ..tile_width as usize {
    //                 let col = tile % cols;
    //                 let row = tile / cols;
    //                 let tile_x = col * tile_width as usize;
    //                 let tile_y = row * tile_height as usize;
    //                 let dest_px = (S::ATLAS_WIDTH  * (tile_y + y)) + (tile_x + x);
    //                 self.pixels[dest_px] = data[cursor()];
    //                 #[cfg(std)]{ pix_count += 1; }
    //             }
    //         }
    //     }
    //     #[cfg(std)]{ println!("{} pixels loaded", pix_count); }

    //     // Load fonts
    //     for _ in 0 .. font_count {
    //         let id = data[cursor()];
    //         let start = data[cursor()];
    //         let len = data[cursor()]; 
    //         // if id as usize != self.fonts.len() { panic!("Atlas Error: Font ID does not match its Pool index!")}
    //         let font = Font { start, len, id, tileset:result };
    //         #[cfg(std)]{ println!("    Loading font:{:?}", font); }
    //         self.fonts[id as usize] = Some(font);
    //     }

    //     // Load Anims
    //     for _ in 0 .. anim_count {
    //         let id = data[cursor()];
    //         let group = data[cursor()];
    //         let fps = data[cursor()];
    //         let len = data[cursor()];
    //         #[cfg(std)]{ println!("    Loading anim {} with group {}", id, group); }
    //         self.anims[id as usize] = Some(
    //             Anim {
    //                 id,
    //                 group,
    //                 fps,
    //                 len,
    //                 frames: core::array::from_fn(|_frame|{
    //                     #[cfg(std)]{ print!("        Loading frame {} ", _frame); }
    //                     Frame {
    //                         cols: data[cursor()],
    //                         rows: data[cursor()],
    //                         tiles: core::array::from_fn(|_tile|{
    //                             let index = data[cursor()];
    //                             #[cfg(std)]{
    //                                 print!("    tile {}:{},", tile, index);
    //                                 if _tile == ANIM_TILES_PER_FRAME-1 { println!("") }
    //                             }
    //                             Tile {
    //                                 index,
    //                                 flags: data[cursor()]
    //                             }
    //                         })
    //                     }
    //                 }),
    //                 tileset: result,
    //             }
    //         );
    //     }

    //     // Load Tilemaps
    //     for _ in 0 .. tilemap_count {
    //         let id = data[cursor()];
    //         let cols = u16::from_ne_bytes([data[cursor()], data[cursor()]]);
    //         let rows = u16::from_ne_bytes([data[cursor()], data[cursor()]]);
    //         #[cfg(std)]{ println!("    Loading map {} with {}x{} tiles", id, cols, rows); }
    //         self.tilemaps[id as usize] = Some(
    //             Tilemap{
    //                 id,
    //                 tileset: result,
    //                 cols,
    //                 rows,
    //                 bg_buffers: Default::default(),
    //                 tiles: core::array::from_fn(|_|{
    //                     Tile{
    //                         index: data[cursor()],
    //                         flags: data[cursor()],
    //                     }
    //                 }
    //             )
    //         });

    //     }
        
        
    //     // Finish tileset insertion
    //     if offset != data.len() {
    //         panic!("Atlas error: expected file length is {}, found {}", data.len(), offset)
    //     }

    //     self.next_tileset += 1;
    //     self.next_free_tile += len;
    //     result
    // }


    // pub fn remove_tileset(&mut self, id:TilesetID) {
    //     if let Some(tileset) = self.tilesets.get(id){
    //         self.next_free_tile -= tileset.len;
    //         self.tilesets.remove(id);
    //     }
    // }


    // TODO: Return option
    pub fn get_tileset(&self, id:usize) -> &Tileset {
        &self.tilesets[id].as_ref().unwrap()
    }


    // TODO: Return option
    pub fn get_tile_and_palette(&self, index:u8, tileset_id:usize) -> (TileID, &Palette<S>) {
        let tileset = &self.tilesets[tileset_id].as_ref().unwrap();
        let palette = self.palettes[tileset.palette_id as usize].as_ref().unwrap();
        (TileID(tileset.start_index + index as u16), palette)
    }


    pub fn get_rect(&self, index:usize) -> Rect<u8> {
        self.rects[index]
    }


    pub fn get_pixel(&self, x:usize, y:usize) -> u8 {
        let index = (y * S::ATLAS_WIDTH) + x;
        self.pixels[index]
    }


    pub fn get_anims(&self) -> &[Option<Anim>; S::ANIM_COUNT] { &self.anims }

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