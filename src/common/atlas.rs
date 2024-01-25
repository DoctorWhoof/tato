use core::array;

use crate::*;

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
    pub(crate) rects:[Rect<u8>; S::ATLAS_TILE_COUNT],
    pub(crate) fonts: [Font; S::FONT_COUNT],
    pub(crate) anims: [Anim; S::ANIM_COUNT],
    pub(crate) tilemaps: [Tilemap; S::TILEMAP_COUNT],
    pub(crate) palettes: [Palette<S>; S::PALETTE_COUNT],
    pub(crate) tilesets: [Tileset; S::TILESET_COUNT],
    
    pixels:[u8; S::ATLAS_WIDTH * S::ATLAS_HEIGHT],
    next_tileset:u16,
    next_free_tile:u16,
}


impl<S:Specs> Atlas<S>
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
    pub(crate) fn new() -> Self {
        #[cfg(feature = "std")]{
            println!("Atlas: Creating new Atlas with {} tiles.", S::ATLAS_TILE_COUNT);
        }
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
                    w:S::TILE_WIDTH,
                    h:S::TILE_HEIGHT
                }
            }),
            fonts: core::array::from_fn( |_| Default::default() ),
            anims: core::array::from_fn( |_| Default::default() ),
            tilemaps: core::array::from_fn( |_| Default::default() ),
            palettes: core::array::from_fn( |i| Palette::new( u8::try_from(i).unwrap() )),
            // tilesets: core::array::from_fn( |_| Default::default() ),
            tilesets: core::array::from_fn( |_| Default::default() ),
            next_tileset: 0,
            next_free_tile: 0,
        }
    }


    pub fn width(&self) -> usize { S::ATLAS_WIDTH }


    pub fn height(&self) -> usize { S::ATLAS_HEIGHT }


    pub fn tile_width(&self) -> u8 { S::TILE_WIDTH }


    pub fn tile_height(&self) -> u8 { S::TILE_HEIGHT }


    pub fn insert_tileset( &mut self, tileset_id:impl ByteID, data:&[u8] ) {
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

        // TODO: Move as much error checking to build stage, create a checksum of something like that
        // Wrap up header, error checking
        let tile_len = (S::TILE_WIDTH * S::TILE_HEIGHT) as u16;
        let tile_count = pixel_count / tile_len;
        let tile_length = tile_width as u16 * tile_height as u16;

        if (tile_width != S::TILE_WIDTH) || (tile_height != S::TILE_HEIGHT) {
            panic!("Atlas error: invalid tileset dimensions. Expected {}x{} tiles, found {}x{}",
            S::TILE_WIDTH, S::TILE_HEIGHT, tile_width, tile_height)
        }

        if tile_count as usize > S::ATLAS_TILE_COUNT {
            panic!("Atlas error: invalid tile count. Expected a maximum of {} tiles, found {}",
            S::ATLAS_TILE_COUNT, tile_count)
        }

        if tile_count as usize * tile_length as usize != pixel_count as usize {
            panic!("Atlas error: invalid tileset dimensions. Expected {} pixels.", pixel_count)
        }
        
        // Insert new tileset
        #[cfg(feature = "std")]{
            println!(
                "\nInserting tileset {:#?} with {} pixels, {} colors, {} fonts, {} anims, and {} maps",
                tileset_id.to_u8(), pixel_count, palette_len, font_count, anim_count, tilemap_count
            );
        };
        let start_index = self.next_free_tile; 
        let len = pixel_count / tile_len; 
        #[cfg(feature = "std")]{
            println!("    start_index:{}, len:{} , palette_id:{}", start_index, len, palette_id);
        }
        self.tilesets[tileset_id.to_usize()] = Tileset {
            unique_id:tileset_id.to_u8(),
            start_index,
            len,
            palette_id
        };

        // Load palettte. Will overwrite the same palette sometimes, since tilesets may use the same palette
        let mut palette = Palette::new(palette_id);
        #[cfg(feature = "std")]{
            println!("    Loading palette {} with {} colors", palette_id, palette_len);
        }
        // let current = offset.clone();
        // println!("    Cursor: {}", current);
        for _i in 0 .. palette_len {
            let r = data[cursor()];
            let g = data[cursor()];
            let b = data[cursor()];
            let a = data[cursor()];
            palette.push(Color{r,g,b,a});
            #[cfg(feature = "std")]{
                println!("        {:02}: {:?}", _i, Color{r,g,b,a});
            }
        }
        
        self.palettes[palette_id as usize] = palette;

        // Load pixels from linear format into tile-formatted.
        let mut _pix_count:usize = 0;
        let cols = S::ATLAS_WIDTH  / S::TILE_WIDTH as usize;
        for tile in  start_index as usize .. (start_index + len) as usize {
            for y in 0 .. tile_height as usize {
                for x in 0 ..tile_width as usize {
                    let col = tile % cols;
                    let row = tile / cols;
                    let tile_x = col * tile_width as usize;
                    let tile_y = row * tile_height as usize;
                    let dest_px = (S::ATLAS_WIDTH  * (tile_y + y)) + (tile_x + x);
                    self.pixels[dest_px] = data[cursor()];
                    _pix_count += 1;
                }
            }
        }
        #[cfg(feature = "std")]{ println!("{} pixels loaded", _pix_count); }

        // Load fonts
        for _ in 0 .. font_count {
            let id = data[cursor()];
            let start = data[cursor()];
            let len = data[cursor()]; 
            // if id as usize != self.fonts.len() { panic!("Atlas Error: Font ID does not match its Pool index!")}
            let font = Font { start, len, id, tileset:tileset_id.to_u8() };
            #[cfg(feature = "std")]{ println!("    Loading font:{:?}", font); }
            self.fonts[id as usize] = font;
        }

        // Load Anims
        for _ in 0 .. anim_count {
            let id = data[cursor()];
            let group = data[cursor()];
            let fps = data[cursor()];
            let len = data[cursor()];
            #[cfg(feature = "std")]{ println!("    Loading anim {} with group {}", id, group); }
            self.anims[id as usize] = Anim {
                id,
                group,
                fps,
                len,
                frames: core::array::from_fn(|_frame|{
                    #[cfg(feature = "std")]{ print!("        Loading frame {} ", _frame); }
                    Frame {
                        cols: data[cursor()],
                        rows: data[cursor()],
                        tiles: core::array::from_fn(|_tile|{
                            let index = data[cursor()];
                            // #[cfg(feature = "std")]{
                            //     print!("    tile {}:{},", tile, index);
                            //     if _tile == ANIM_TILES_PER_FRAME-1 { println!("") }
                            // }
                            Tile {
                                index,
                                flags: data[cursor()]
                            }
                        })
                    }
                }),
                tileset: tileset_id.to_u8(),
            };
        }

        // Load Tilemaps
        for _ in 0 .. tilemap_count {
            let id = data[cursor()];
            let cols = u16::from_ne_bytes([data[cursor()], data[cursor()]]);
            let rows = u16::from_ne_bytes([data[cursor()], data[cursor()]]);
            #[cfg(feature = "std")]{ println!("    Loading map {} with {}x{} tiles", id, cols, rows); }
            self.tilemaps[id as usize] = Tilemap{
                id,
                tileset: tileset_id.to_u8(),
                cols,
                rows,
                bg_buffers: Default::default(),
                tiles: core::array::from_fn(|_|{
                    Tile{
                        index: data[cursor()],
                        flags: data[cursor()],
                    }
                }),
            };

        }
        
        
        // Finish tileset insertion
        if offset != data.len() {
            panic!("Atlas error: expected file length is {}, found {}", data.len(), offset)
        }

        self.next_tileset += 1;
        self.next_free_tile += len;
    }


    // pub fn remove_tileset(&mut self, id:TilesetID) {
    //     self.tilesets.remove(id);
    // }


    pub fn get_tileset(&self, id:usize) -> &Tileset {
        &self.tilesets[id]
    }


    pub fn get_tile_and_palette(&self, index:u8, tileset_id:usize) -> (TileID, &Palette<S>) {
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