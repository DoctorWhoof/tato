use core::array;

use crate::*;
use slotmap::SlotMap;


pub struct Atlas<const PIXEL_COUNT:usize, const TILE_COUNT:usize> {
    pixels:[u8; PIXEL_COUNT],
    rects:[Rect<u8>; TILE_COUNT],
    next_tileset:u16,
    next_free_tile:u16,
    tilesets: SlotMap<TilesetID, Tileset>,
    width: usize,
    height:usize,
    tile_width:u8,
    tile_height:u8,
}


impl<const PIXEL_COUNT:usize, const TILE_COUNT:usize>
Atlas<PIXEL_COUNT, TILE_COUNT>  {

    pub fn new(width:usize, height:usize, tile_width:u8, tile_height:u8) -> Self {
        // println!("Atlas: Creating new Atlas with {} tiles.", MAX_TILES);
        assert!(PIXEL_COUNT==width*height, "Atlas: Error, width x height must equal PIXEL_COUNT");
        assert!(TILE_COUNT==(width/tile_width as usize)*(height/tile_height as usize), "Atlas: Invalid tile count.");
        Atlas {
            pixels: [0; PIXEL_COUNT],
            rects: array::from_fn( |i| {
                // generates all tiles
                let tile_x = i * tile_width as usize;
                let x = (tile_x % width) as u8;
                let y = ((tile_x / width) * tile_height as usize) as u8;
                Rect{ x ,y , w:tile_width, h:tile_height }
            }),
            tilesets: Default::default(),
            next_tileset: 0,
            next_free_tile: 0,
            width,
            height,
            tile_width,
            tile_height
        }
    }


    pub fn width(&self) -> usize { self.width }


    pub fn height(&self) -> usize { self.height }


    pub fn tile_width(&self) -> u8 { self.tile_width }


    pub fn tile_height(&self) -> u8 { self.tile_height }


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
        let tile_len = (TILE_WIDTH * TILE_HEIGHT) as u16;
        let tile_count = pixel_count / tile_len;
        let tile_length = tile_width as u16 * tile_height as u16;
        if (tile_count * tile_length != pixel_count) || (tile_width != self.tile_width) || (tile_height != self.tile_height) {
            panic!(
                "Atlas error: invalid tileset dimensions. Expected {} pixels with ({}x{}) tiles",
                pixel_count, TILE_WIDTH, TILE_HEIGHT
            )
        }
        
        // Insert new tileset
        let start_index = self.next_free_tile; 
        let len = pixel_count / tile_len; 
        let result = self.tilesets.insert_with_key(|key| {
            Tileset { unique_id:key, start_index, len }
        });

        // Loads from linear-formatted pixels into tile-formatted pixels
        let cols = self.width / self.tile_width as usize;
        let mut source_px = 0;
        for tile in  start_index as usize .. (start_index + len) as usize {
            for y in 0 .. tile_height as usize {
                for x in 0 ..tile_width as usize {
                    let col = tile % cols;
                    let row = tile / cols;
                    let tile_x = col * tile_width as usize;
                    let tile_y = row * tile_height as usize;
                    let dest_px = (self.width * (tile_y + y)) + (tile_x + x);
                    self.pixels[dest_px] = data[tileset_header_len + source_px];
                    source_px += 1;
                }
            }
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


    pub fn get_tile_from_tileset(&self, index:u8, tileset_id:TilesetID) -> TileID {
        let tileset = &self.tilesets[tileset_id];
        let result = TileID(tileset.start_index + index as u16);
        result
    }


    pub fn get_rect(&self, index:usize) -> Rect<u8> {
        self.rects[index]
    }

    pub fn get_pixel(&self, x:usize, y:usize) -> u8 {
        let index = (y * self.width) + x;
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