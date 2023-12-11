use core::array;

use crate::*;
use slotmap::SlotMap;

pub const ATLAS_WIDTH:usize = 128;
pub const ATLAS_HEIGHT:usize = 128;
pub const ATLAS_PIXEL_COUNT:usize = ATLAS_WIDTH * ATLAS_HEIGHT;

pub const TILE_WIDTH:usize = 8;
pub const TILE_HEIGHT:usize = 8;
pub const TILE_LENGTH:usize = TILE_WIDTH * TILE_HEIGHT;
pub const TILE_COUNT:usize = ATLAS_PIXEL_COUNT/TILE_LENGTH;

pub const TILESET_COUNT:usize = 4;
pub const TILESET_HEADER_TEXT:&str = "tileset_1.0";


pub struct Atlas {
    pub pixels:[u8; ATLAS_PIXEL_COUNT],
    pub rects:[Rect<u8>; TILE_COUNT],
    next_tileset:u16,
    next_free_tile:u16,
    tilesets: SlotMap<TilesetID, Tileset>
}


impl Default for Atlas  {
    fn default() -> Self {
        // println!("Atlas: Creating new Atlas with {} tiles.", MAX_TILES);
        let Ok(tile_width) = u8::try_from(TILE_WIDTH) else { panic!("Tile width can't be more than 255") };
        let Ok(tile_height) = u8::try_from(TILE_HEIGHT) else { panic!("Tile height can't be more than 255") };

        Atlas {
            pixels: [0; ATLAS_PIXEL_COUNT],
            rects: array::from_fn( |i| {
                // generates all tiles
                let tile_x = i * TILE_WIDTH;
                let x = (tile_x % ATLAS_WIDTH) as u8;
                let y = ((tile_x / ATLAS_WIDTH) * TILE_HEIGHT) as u8;
                Rect{ x ,y , w:tile_width, h:tile_height }
            }),
            tilesets: Default::default(),
            next_tileset: 0,
            next_free_tile: 0,
        }
    }
}


impl Atlas {


    pub fn insert_tileset( &mut self, data:&[u8] ) -> TilesetID {
        let mut offset = TILEMAP_HEADER_TEXT.len()-1;
        if data[0 ..= offset] != *TILESET_HEADER_TEXT.as_bytes() { panic!("Atlas error: Invalid .tiles file") }

        let mut cursor = || -> usize {
            offset += 1;
            offset
        };

        let tile_width = data[cursor()];
        let tile_height = data[cursor()];
        let pixel_count = u16::from_ne_bytes([data[cursor()], data[cursor()]]);

        // Wrap up header, error checking
        let tileset_header_len = cursor();
        let tile_count = pixel_count / TILE_LENGTH as u16;
        let tile_length = tile_width as u16 * tile_height as u16;
        if (tile_count * tile_length != pixel_count) || (tile_width as usize != TILE_WIDTH) || (tile_height as usize != TILE_HEIGHT) {
            panic!(
                "Atlas error: invalid tileset dimensions. Expected {} pixels with ({}x{}) tiles",
                pixel_count, TILE_WIDTH, TILE_HEIGHT
            )
        }
        
        // Insert new tileset
        let start_index = self.next_free_tile; 
        let len = pixel_count / TILE_LENGTH as u16; 
        let result = self.tilesets.insert_with_key(|key| {
            Tileset { unique_id:key, start_index, len }
        });

        // Loads from linear-formatted pixels into tile-formatted pixels
        let cols = ATLAS_WIDTH / TILE_WIDTH;
        let mut source_px = 0;
        for tile in  start_index as usize .. (start_index + len) as usize {
            for y in 0 .. tile_height as usize {
                for x in 0 ..tile_width as usize {
                    let col = tile % cols;
                    let row = tile / cols;
                    let tile_x = col * tile_width as usize;
                    let tile_y = row * tile_height as usize;
                    let dest_px = (ATLAS_WIDTH * (tile_y + y)) + (tile_x + x);
                    self.pixels[dest_px] = data[tileset_header_len + source_px];
                    source_px += 1;
                }
            }
        }
        
        self.next_tileset += 1;
        self.next_free_tile += len;
        result
    }


    // #[allow(unused)]
    // pub fn pop_tileset(&mut self) {
    //     if self.next_tileset > 0 {
    //         self.next_tileset -= 1;
    //         let last_tileset = &self.tilesets[self.next_tileset as usize];
    //         self.next_free_tile -= last_tileset.len;
    //     } else {
    //         panic!("Atlas: Error, no tiles left to pop!")
    //     }
    // }


    pub fn get_tileset(&self, id:TilesetID) -> &Tileset {
        // #[cfg(debug_assertions)]{
        //     if id.index >= self.next_tileset {
        //         panic!("Atlas: Invalid tileset ID {:?} (max is {:?})", id, self.next_tileset-1)
        //     }
        // }
        &self.tilesets[id]
    }


    pub fn get_tile_from_tileset(&self, index:u8, tileset_id:TilesetID) -> TileID {
        let tileset = &self.tilesets[tileset_id];
        let result = TileID(tileset.start_index + index as u16);
        // #[cfg(debug_assertions)]{
        //     if tileset_id.unique_id != tileset.unique_id || tileset_id.index >= self.next_tileset{
        //         panic!( "\nAtlas: Error, attempt to use invalid TilesetID:{:?}\n", tileset_id )
        //     }
        //     if index as u16 > tileset.len {
        //         panic!("Atlas: Invalid tileID {:?} (max is {:?})", result, tileset.start_index + tileset.len)
        //     }
        // }
        result
    }


    #[inline]
    pub fn get_pixel(&self, x:usize, y:usize) -> u8 {
        let index = (y * ATLAS_WIDTH) + x;
        self.pixels[index]
    }

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

    // #[inline]
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