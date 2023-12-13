pub const TILEMAP_HEADER_TEXT:&str = "tilemap_1.0" ;
pub const TILEMAP_HEADER_LEN:usize = 15;
pub const TILEMAP_LEN:usize = 48 * 48;

use super::*;
use slotmap::new_key_type;


new_key_type! { pub struct TilemapID; }


#[derive(Default, Clone)]
struct BgBuffer {
    frame:Frame,
    source_col:u16,    // Assuming tilemaps aren't huge, otherwise needs to be a larger int
    source_row:u16,
}


pub struct Tilemap {
    pub id: TilemapID,
    pub cols:u16,
    pub rows:u16,
    pub tiles:[Tile; TILEMAP_LEN],
    pub tileset: TilesetID,
    bg_buffers:[BgBuffer; 1],
    bg_buffer_head:usize,
    bg_buffers_dirty: bool
}



impl Default for Tilemap {
    fn default() -> Self {
        Self {
            id:TilemapID::default(),
            cols:1, rows:1,
            tiles:core::array::from_fn(|_| Tile::default() ),
            tileset: TilesetID::default(),
            bg_buffers: Default::default(),
            bg_buffer_head: 0,
            bg_buffers_dirty: false
        }
    }
}


impl Tilemap {

    pub fn load( data:&[u8], tileset:TilesetID, id:TilemapID ) -> Self {
        let text_len = TILEMAP_HEADER_TEXT.len();
        if data[0 .. text_len] != *TILEMAP_HEADER_TEXT.as_bytes() { panic!("World error: Invalid .tilemap file") }
        if data.len() < TILEMAP_HEADER_LEN + 1 { panic!("World error: Invalid .tilemap file") }

        let cols = u16::from_ne_bytes([data[text_len], data[text_len+1]]);
        let rows = u16::from_ne_bytes([data[text_len+2], data[text_len+3]]);
        
        if cols as usize * rows as usize > TILEMAP_LEN {
            panic!("Tilemap: Error creating {} x {} tilemap, capacity of {} exceeded", cols, rows, TILEMAP_LEN)
        }

        let max_len = (TILEMAP_LEN * 2) + TILEMAP_HEADER_LEN;
        if data.len() > max_len {
            panic!("Tilemap: Error, tilemap data over capacity. Should be less than {}, but it's {}", max_len, data.len())
        }

        let mut tile_data:[Tile; TILEMAP_LEN] = core::array::from_fn(|_| Tile::default() );
        let data_len = (data.len() - TILEMAP_HEADER_LEN) / 2;
        (0 .. data_len).for_each(|i| {
            let index = (i * 2) + TILEMAP_HEADER_LEN;
            tile_data[i].index = data[index];
            tile_data[i].flags = data[index + 1];
        });

        Self {
            id,
            cols, rows,
            tiles: tile_data,
            tileset, 
            bg_buffers: Default::default(),
            bg_buffer_head: 0,
            bg_buffers_dirty: false
        }
    }


    pub fn reset_bg_buffers(&mut self) {
        if self.bg_buffers_dirty {
            for n in 0 .. self.bg_buffer_head {
                let buffer = &self.bg_buffers[n];
                let rows = buffer.frame.rows as usize;
                let columns = buffer.frame.cols as usize;
                for row in 0 .. rows {
                    for col in 0 .. columns {
                        let buffer_index = (row * columns) + col;
                        let tilemap_index = ((row + buffer.source_row as usize) * self.cols as usize) + col + buffer.source_col as usize;
                        self.tiles[tilemap_index] = buffer.frame.tiles[buffer_index];
                        // self.tiles[tilemap_index].index = 0;
                    }
                }
                // println!("Restoring {:?} at {},{}", buffer.frame.tiles[0], buffer.source_col, buffer.source_row);
            }
        }
        self.bg_buffers_dirty = false;
        self.bg_buffer_head = 0;
    }


    pub fn insert_bg_buffer(&mut self, col:u16, row:u16, cols:u8, rows:u8) {
        self.bg_buffers_dirty = true;

        let tile_count = cols as usize * rows as usize;
        self.bg_buffers[self.bg_buffer_head] = BgBuffer {
            frame: Frame {
                cols,
                rows,
                tiles: core::array::from_fn(|i|{
                    if i < tile_count {
                        let frame_col = i % cols as usize;
                        let frame_row = i / cols as usize;
                        let abs_col = col as usize + frame_col;
                        let abs_row = row as usize + frame_row;
                        let tilemap_index = (abs_row * self.cols as usize) + abs_col;
                        self.tiles[tilemap_index]
                    } else {
                        Tile::default()
                    }
                })
            },
            source_col: col,
            source_row: row,
        };
        
        self.bg_buffer_head += 1;
        // println!("Inserting bg_buffer at {},{}, total: {}", col, row, self.bg_buffer_head);
    }



    #[inline]
    fn get_index(&self, col:u16, row:u16) -> usize {
        #[cfg(debug_assertions)]
        if col >= self.cols || row >= self.rows {
            panic!("Invalid tilemap coordinates")
        }
        (row as usize * self.cols as usize) + col as usize
    }


    #[allow(unused)]
    pub fn set_tile_index(&mut self, col:u16, row:u16, value:u8) {
        self.tiles[ self.get_index(col, row) ].index = value;
    }


    pub fn get_tile(&self, col:u16, row:u16) -> Tile {
        let index = self.get_index(col, row); 
        self.tiles[index]
    }


    pub fn width(&self, tile_width:usize) -> usize { self.cols as usize * tile_width }


    pub fn height(&self, tile_height:usize) -> usize { self.rows as usize * tile_height }

    
}