use core::array;
use crate::*;
use core::mem::variant_count;

/// Loads and stores fixed size tiles organized into tilesets that can be added and removed individually.
pub struct Atlas<
    S:Specs,
    TilesetEnum:Into<u8> + Copy,
    PaletteEnum:Into<u8> + Copy,
>
where
    [(); variant_count::<TilesetEnum>()]: Sized,
    [(); variant_count::<PaletteEnum>()]: Sized,
    [(); S::ATLAS_WIDTH * S::ATLAS_HEIGHT]: Sized,                                                          //Pixel count
    [(); (S::ATLAS_WIDTH * S::ATLAS_HEIGHT)/(S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)]: Sized,     //Tile count
    [(); S::RENDER_WIDTH * S::RENDER_HEIGHT]: Sized,
    [(); S::ANIM_COUNT]: Sized,
    [(); S::FONT_COUNT]: Sized,
    [(); S::TILEMAP_COUNT]: Sized,
    [(); S::COLORS_PER_PALETTE]: Sized,
{
    // pub(crate) tilesets: [Tileset; S::TILESET_COUNT],    
    // pub(crate) tilesets: [Tileset<S>; variant_count::<TilesetEnum>()],    
    pub(crate) tilesets: [Tileset; variant_count::<TilesetEnum>()],
    pub(crate) palettes: [Palette<S>; variant_count::<PaletteEnum>()],
    pub(crate) fonts: [Font; S::FONT_COUNT],
    pub(crate) anims: [Anim; S::ANIM_COUNT],
    pub(crate) tilemaps: [Tilemap; S::TILEMAP_COUNT],
    pub(crate) rects:[Rect<u8>; (S::ATLAS_WIDTH * S::ATLAS_HEIGHT)/(S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)],
    pixels:[u8; S::ATLAS_WIDTH * S::ATLAS_HEIGHT],
}


impl<
    S:Specs,
    TilesetEnum:Into<u8> + Copy,
    PaletteEnum:Into<u8> + Copy,
    // AnimEnum:Into<u8> + Copy,
> Atlas<S, TilesetEnum, PaletteEnum>
where
    [(); variant_count::<TilesetEnum>()]: Sized,
    [(); variant_count::<PaletteEnum>()]: Sized,
    [(); S::ATLAS_WIDTH * S::ATLAS_HEIGHT]: Sized,  // Pixel count
    [(); (S::ATLAS_WIDTH * S::ATLAS_HEIGHT)/(S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)]: Sized, //Tile count
    [(); S::ANIM_COUNT]: Sized,
    [(); S::FONT_COUNT]: Sized,
    [(); S::TILEMAP_COUNT]: Sized,
    [(); S::COLORS_PER_PALETTE]: Sized,
    [(); S::RENDER_WIDTH * S::RENDER_HEIGHT]: Sized
{
    pub(crate) fn new() -> Self {
        
        #[cfg(feature = "std")]{
            let tile_count = (S::ATLAS_WIDTH * S::ATLAS_HEIGHT) / (S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize);
            println!("Atlas: Creating new Atlas with {} tiles.", tile_count);
        }

        Atlas {
            pixels: [0; S::ATLAS_WIDTH * S::ATLAS_HEIGHT],
            // Assets are always initialized, never an option! The length of the array containing each asset
            // is determined by the Enum associated with it, and the builder script will fail if any
            // enum variant is not initialized
            tilesets: core::array::from_fn( |i| Tileset::new( u8::try_from(i).unwrap() )),
            palettes: core::array::from_fn( |i| Palette::new( u8::try_from(i).unwrap() )),
            fonts: core::array::from_fn( |i| Font::new( u8::try_from(i).unwrap() )),
            anims: core::array::from_fn( |i| Anim::new( u8::try_from(i).unwrap() ) ),
            tilemaps: core::array::from_fn( |i| Tilemap::new( u8::try_from(i).unwrap() ) ),
            rects: array::from_fn( |i| {
                // generates all tiles
                let tile_x = i * S::TILE_WIDTH as usize;
                let x = (tile_x % S::ATLAS_WIDTH) as u8;
                let y = ((tile_x / S::ATLAS_WIDTH) * S::TILE_HEIGHT as usize) as u8;
                Rect{
                    x,
                    y,
                    w:S::TILE_WIDTH,
                    h:S::TILE_HEIGHT
                }
            }),
        }
    }


    pub fn load(&mut self, bytes:&[u8]) {
        let mut cursor = Cursor::new(bytes);

        for letter in ATLAS_HEADER_TEXT.as_bytes() {
            assert!(*letter== cursor.next(), "Atlas Error: Invalid header.")
        }

        // Header data
        assert!(S::TILE_WIDTH ==  cursor.next(), "Atlas Error: Tile width does not match.");                                // Tile Width
        assert!(S::TILE_HEIGHT ==  cursor.next(), "Atlas Error: Tile width does not match.");                               // Tile Height
        assert!(variant_count::<PaletteEnum>() as u8 ==  cursor.next(), "Atlas Error: Palette count does not match");       // Palette Count
        assert!(S::FONT_COUNT as u8 ==  cursor.next(), "Atlas Error: Font count does not match");                           // Font Count
        assert!(S::ANIM_COUNT as u8 ==  cursor.next(), "Atlas Error: Anim count does not match");                           // Anim Count
        assert!(S::TILEMAP_COUNT as u8 ==  cursor.next(), "Atlas Error: Tilemap count does not match");                     // Tilemap Count
        assert!(
            variant_count::<TilesetEnum>() == cursor.next() as usize,
            "Atlas Error: Tileset count does not match"
        );       // Tileset Count

        // Palettes
        for palette in self.palettes.iter_mut(){
            palette.id = cursor.next();
            for color in palette.colors.iter_mut() {
                color.r =  cursor.next();
                color.g =  cursor.next();
                color.b =  cursor.next();
                color.a =  cursor.next();
            }
            #[cfg(feature = "std")]{ println!(" Initializing Palette {}", palette.id); }
        }

        // Fonts
        for font in self.fonts.iter_mut() {
            font.deserialize(&mut cursor);
            #[cfg(feature = "std")]{ println!(" Initializing Font {}", font.id); }
        }

        // Anims
        for anim in self.anims.iter_mut() {
            anim.deserialize(&mut cursor);
            #[cfg(feature = "std")]{ println!(" Initializing Anim {}", anim.id); }
        }

        // Tilemaps
        for tilemap in self.tilemaps.iter_mut() {
            tilemap.deserialize(&mut cursor);
            #[cfg(feature = "std")]{ println!(" Initializing Tilemap {}", tilemap.id); }
        }

        // Tilesets
        for (_tileset_idx,tileset) in self.tilesets.iter_mut().enumerate() {
            // Header text
            for letter in TILESET_HEADER_TEXT.as_bytes() {
                assert!(*letter == cursor.next(), "Atlas Error: Invalid tileset header." )
            }
            
            //TODO: Check actual amount of available pixels
            let pixel_count = u16::from_ne_bytes([cursor.next(), cursor.next()]);
            assert!((pixel_count as usize) < (S::ATLAS_WIDTH * S::ATLAS_HEIGHT),  "Atlas error: Tileset pixels count will overflow!");
            
            // Fields
            tileset.start_index = u16::from_ne_bytes([cursor.next(), cursor.next()]);
            tileset.len = u16::from_ne_bytes([cursor.next(), cursor.next()]);
            tileset.palette_id = cursor.next();
            
            // Pixels
            #[cfg(feature = "std")]{ print!("  Initializing tileset {} ... ", _tileset_idx); }

            // Load pixels from linear format into tile-formatted.
            let mut _pix_count:usize = 0;
            let cols = S::ATLAS_WIDTH / S::TILE_WIDTH as usize;
            for tile_index in tileset.start_index as usize .. (tileset.start_index + tileset.len) as usize {
                for y in 0 .. S::TILE_HEIGHT as usize {
                    for x in 0 ..S::TILE_WIDTH as usize {
                        let col = tile_index % cols;
                        let row = tile_index / cols;
                        let tile_x = col * S::TILE_WIDTH as usize;
                        let tile_y = row * S::TILE_HEIGHT as usize;
                        let dest_px = (S::ATLAS_WIDTH  * (tile_y + y)) + (tile_x + x);
                        self.pixels[dest_px] = cursor.next();
                        _pix_count += 1;
                    }
                }
            }
            #[cfg(feature = "std")]{ println!("{} pixels loaded", _pix_count); }

        }
    }


    pub fn width(&self) -> usize { S::ATLAS_WIDTH }


    pub fn height(&self) -> usize { S::ATLAS_HEIGHT }


    pub fn tile_width(&self) -> u8 { S::TILE_WIDTH }


    pub fn tile_height(&self) -> u8 { S::TILE_HEIGHT }


    // pub fn insert_tileset( &mut self, tileset_id:impl IntoPrimitive, data:&[u8] ) {
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

    //     // TODO: Move as much error checking to build stage, create a checksum of something like that
    //     // Wrap up header, error checking
    //     let tile_len = (S::TILE_WIDTH * S::TILE_HEIGHT) as u16;
    //     let tile_count = pixel_count / tile_len;
    //     let tile_length = tile_width as u16 * tile_height as u16;

    //     if (tile_width != S::TILE_WIDTH) || (tile_height != S::TILE_HEIGHT) {
    //         panic!("Atlas error: invalid tileset dimensions. Expected {}x{} tiles, found {}x{}",
    //         S::TILE_WIDTH, S::TILE_HEIGHT, tile_width, tile_height)
    //     }

    //     if tile_count as usize > S::ATLAS_TILE_COUNT {
    //         panic!("Atlas error: invalid tile count. Expected a maximum of {} tiles, found {}",
    //         S::ATLAS_TILE_COUNT, tile_count)
    //     }

    //     if tile_count as usize * tile_length as usize != pixel_count as usize {
    //         panic!("Atlas error: invalid tileset dimensions. Expected {} pixels.", pixel_count)
    //     }
        
    //     // Insert new tileset
    //     #[cfg(feature = "std")]{
    //         println!(
    //             "\nInserting tileset {:#?} with {} pixels, {} colors, {} fonts, {} anims, and {} maps",
    //             tileset_id.into(), pixel_count, palette_len, font_count, anim_count, tilemap_count
    //         );
    //     };
    //     let start_index = self.next_free_tile; 
    //     let len = pixel_count / tile_len; 
    //     #[cfg(feature = "std")]{
    //         println!("    start_index:{}, len:{} , palette_id:{}", start_index, len, palette_id);
    //     }
    //     self.tilesets[tileset_id.to_usize()] = Tileset {
    //         unique_id:tileset_id.into(),
    //         start_index,
    //         len,
    //         palette_id
    //     };

    //     // Load palettte. Will overwrite the same palette sometimes, since tilesets may use the same palette
    //     let mut palette = Palette::new(palette_id);
    //     #[cfg(feature = "std")]{
    //         println!("    Loading palette {} with {} colors", palette_id, palette_len);
    //     }
    //     // let current = offset.clone();
    //     // println!("    Cursor: {}", current);
    //     for _i in 0 .. palette_len {
    //         let r = data[cursor()];
    //         let g = data[cursor()];
    //         let b = data[cursor()];
    //         let a = data[cursor()];
    //         palette.push(Color{r,g,b,a});
    //         #[cfg(feature = "std")]{
    //             println!("        {:02}: {:?}", _i, Color{r,g,b,a});
    //         }
    //     }
        
    //     self.palettes[palette_id as usize] = palette;

    //     // Load pixels from linear format into tile-formatted.
    //     let mut _pix_count:usize = 0;
    //     let cols = S::ATLAS_WIDTH  / S::TILE_WIDTH as usize;
    //     for tile in  start_index as usize .. (start_index + len) as usize {
    //         for y in 0 .. tile_height as usize {
    //             for x in 0 ..tile_width as usize {
    //                 let col = tile % cols;
    //                 let row = tile / cols;
    //                 let tile_x = col * tile_width as usize;
    //                 let tile_y = row * tile_height as usize;
    //                 let dest_px = (S::ATLAS_WIDTH  * (tile_y + y)) + (tile_x + x);
    //                 self.pixels[dest_px] = data[cursor()];
    //                 _pix_count += 1;
    //             }
    //         }
    //     }
    //     #[cfg(feature = "std")]{ println!("{} pixels loaded", _pix_count); }

    //     // Load fonts
    //     for _ in 0 .. font_count {
    //         let id = data[cursor()];
    //         let start = data[cursor()];
    //         let len = data[cursor()]; 
    //         // if id as usize != self.fonts.len() { panic!("Atlas Error: Font ID does not match its Pool index!")}
    //         let font = Font { start, len, id, tileset:tileset_id.into() };
    //         #[cfg(feature = "std")]{ println!("    Loading font:{:?}", font); }
    //         self.fonts[id as usize] = font;
    //     }

    //     // Load Anims
    //     for _ in 0 .. anim_count {
    //         let id = data[cursor()];
    //         let group = data[cursor()];
    //         let fps = data[cursor()];
    //         let len = data[cursor()];
    //         #[cfg(feature = "std")]{ println!("    Loading anim {} with group {}", id, group); }
    //         self.anims[id as usize] = Anim {
    //             id,
    //             group,
    //             fps,
    //             len,
    //             frames: core::array::from_fn(|_frame|{
    //                 #[cfg(feature = "std")]{ print!("        Loading frame {} ", _frame); }
    //                 Frame {
    //                     cols: data[cursor()],
    //                     rows: data[cursor()],
    //                     tiles: core::array::from_fn(|_tile|{
    //                         let index = data[cursor()];
    //                         // #[cfg(feature = "std")]{
    //                         //     print!("    tile {}:{},", tile, index);
    //                         //     if _tile == ANIM_TILES_PER_FRAME-1 { println!("") }
    //                         // }
    //                         Tile {
    //                             index,
    //                             flags: data[cursor()]
    //                         }
    //                     })
    //                 }
    //             }),
    //             tileset: tileset_id.into(),
    //         };
    //     }

    //     // Load Tilemaps
    //     for _ in 0 .. tilemap_count {
    //         let id = data[cursor()];
    //         let cols = u16::from_ne_bytes([data[cursor()], data[cursor()]]);
    //         let rows = u16::from_ne_bytes([data[cursor()], data[cursor()]]);
    //         #[cfg(feature = "std")]{ println!("    Loading map {} with {}x{} tiles", id, cols, rows); }
    //         self.tilemaps[id as usize] = Tilemap{
    //             id,
    //             tileset: tileset_id.into(),
    //             cols,
    //             rows,
    //             bg_buffers: Default::default(),
    //             tiles: core::array::from_fn(|_|{
    //                 Tile{
    //                     index: data[cursor()],
    //                     flags: data[cursor()],
    //                 }
    //             }),
    //         };

    //     }
        
        
    //     // Finish tileset insertion
    //     if offset != data.len() {
    //         panic!("Atlas error: expected file length is {}, found {}", data.len(), offset)
    //     }

    //     self.next_tileset += 1;
    //     self.next_free_tile += len;
    // }


    // pub fn remove_tileset(&mut self, id:TilesetID) {
    //     self.tilesets.remove(id);
    // }


    // pub fn get_tileset(&self, id:usize) -> &Tileset<S> {
    pub fn get_tileset(&self, id:usize) -> &Tileset {
        &self.tilesets[id]
    }


    // pub fn get_tile_and_palette(&self, index:u8, tileset_id:usize) -> (TileID, &Palette<S>) {
    //     let tileset = &self.tilesets[tileset_id];
    //     let palette = self.palettes.get(tileset.palette_id as usize).unwrap();
    //     (TileID(tileset.start_index + index as u16), palette)
    // }

    pub fn get_tile(&self, index:u8, tileset_id:usize) -> TileID {
        let tileset = &self.tilesets[tileset_id];
        TileID(tileset.start_index + index as u16)
    }


    pub fn get_rect(&self, index:usize) -> Rect<u8> {
        self.rects[index]
    }


    pub fn get_pixel(&self, x:usize, y:usize) -> u8 {
        let index = (y * S::ATLAS_WIDTH) + x;
        self.pixels[index]
    }


    pub fn get_anims(&self) -> &[Anim; S::ANIM_COUNT] { &self.anims }

    
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

    // Just some old code

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


    
}