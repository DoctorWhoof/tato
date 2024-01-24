use crate::EntityID;

use crate::*;
use slotmap::SecondaryMap;

// slotmap::new_key_type! {
//     /// A key to the World slotmap containing tilemaps.
//     pub struct TilemapID;
// }

/// A rectangular array of tiles that belong to a single Tileset. Also provides "BgBuffers" so that
/// AnimTiles can restore the BG contents they overwrite.
pub struct Tilemap {
    pub id: u8,
    pub tileset: u8,
    pub cols:u16,
    pub rows:u16,
    pub bg_buffers:SecondaryMap<EntityID, BgBuffer>,
    pub tiles:[Tile; TILEMAP_LEN],
}


impl Default for Tilemap {
    fn default() -> Self {
        Self {
            id:0,
            tileset: 0,
            cols:1, rows:1,
            bg_buffers: Default::default(),
            tiles:core::array::from_fn(|_| Tile::default() ),
        }
    }
}


impl Tilemap {

    // pub fn id(&self) -> u8 { self.id }


    // pub fn cols(&self) -> u16 { self.cols }


    // pub fn rows(&self) -> u16 { self.rows }


    // pub fn tiles(&self) -> &[Tile; TILEMAP_LEN] { &self.tiles }


    pub fn new(tiles:[Tile; TILEMAP_LEN], id:u8, cols:u16, rows:u16) -> Self {
        Self { id, cols, rows, tiles, .. Default::default() }
    }


    pub fn bg_buffer_count(&self) -> usize { self.bg_buffers.len() }


    // pub fn load( data:&[u8], tileset:TilesetID, id:TilemapID ) -> Self {
    //     let text_len = TILEMAP_HEADER_TEXT.len();
    //     if data[0 .. text_len] != *TILEMAP_HEADER_TEXT.as_bytes() { panic!("World error: Invalid .tilemap file") }
    //     if data.len() < TILEMAP_HEADER_LEN + 1 { panic!("World error: Invalid .tilemap file") }

    //     let cols = u16::from_ne_bytes([data[text_len], data[text_len+1]]);
    //     let rows = u16::from_ne_bytes([data[text_len+2], data[text_len+3]]);
        
    //     if cols as usize * rows as usize > TILEMAP_LEN {
    //         panic!("Tilemap: Error creating {} x {} tilemap, capacity of {} exceeded", cols, rows, TILEMAP_LEN)
    //     }

    //     let max_len = (TILEMAP_LEN * 2) + TILEMAP_HEADER_LEN;
    //     if data.len() > max_len {
    //         panic!("Tilemap: Error, tilemap data over capacity. Should be less than {}, but it's {}", max_len, data.len())
    //     }

    //     let mut tile_data:[Tile; TILEMAP_LEN] = core::array::from_fn(|_| Tile::default() );
    //     let data_len = (data.len() - TILEMAP_HEADER_LEN) / 2;
    //     (0 .. data_len).for_each(|i| {
    //         let index = (i * 2) + TILEMAP_HEADER_LEN;
    //         tile_data[i].index = data[index];
    //         tile_data[i].flags = data[index + 1];
    //     });

    //     Self {
    //         id,
    //         cols, rows,
    //         tiles: tile_data,
    //         tileset, 
    //         bg_buffers: Default::default(),
    //     }
    // }


    pub fn restore_bg_buffer(&mut self, id:EntityID) {
        let Some(buffer) = self.bg_buffers.get(id) else { return };

        let rows = buffer.frame.rows as usize;
        let columns = buffer.frame.cols as usize;
        for row in 0 .. rows {
            for col in 0 .. columns {
                let buffer_index = (row * columns) + col;
                let tilemap_index = ((row + buffer.source_row as usize) * self.cols as usize) + col + buffer.source_col as usize;
                self.tiles[tilemap_index] = buffer.frame.tiles[buffer_index];
            }
        }
    }

    // Returns false if off screen
    pub fn store_bg_buffer(&mut self, col:i32, row:i32, cols:u8, rows:u8, id:EntityID) -> bool {
        // If this buffer already wrote to the tilemap on last frame, restore it before moving on
        self.restore_bg_buffer(id);

        if col <= -(cols as i32) || row <= -(rows as i32) { return false }
        if col >= self.cols as i32 || row >= self.rows as i32 { return false }

        let Ok(col) = u16::try_from(col) else { return false };
        let Ok(row) = u16::try_from(row) else { return false };

        let tile_count = cols as usize * rows as usize;
        self.bg_buffers.insert(id, BgBuffer {
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
        });
        true
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


    pub fn width(&self, tile_width:u8) -> usize { self.cols as usize * tile_width as usize }


    pub fn height(&self, tile_height:u8) -> usize { self.rows as usize * tile_height as usize }

    
}