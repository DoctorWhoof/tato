use core::mem::size_of;
// use libm::fabsf;
use slotmap::SecondaryMap;
use crate::{BgBuffer, ByteArray, Cursor, EntityID, Frame, Tile, Vec2, TILEMAP_LEN};

const SIZE_OF_TILEMAP:usize = 7 + (size_of::<Tile>() * TILEMAP_LEN); // id, tileset, cols(2 bytes), rows(2 bytes), palette, [tiles] 

/// A rectangular array of tiles that belong to a single Tileset. Also provides "BgBuffers" so that
/// BgTiles can restore the BG contents they overwrite.
pub struct Tilemap {
    pub id: u8,
    pub tileset: u8,
    pub cols:u16,
    pub rows:u16,
    pub palette:u8,
    pub tiles:[Tile; TILEMAP_LEN],
    pub bg_buffers:SecondaryMap<EntityID, BgBuffer>,
}


impl Clone for Tilemap {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            tileset: self.tileset,
            cols: self.cols,
            rows: self.rows,
            palette: self.palette,
            tiles: self.tiles,
            bg_buffers: Default::default(), // no need to clone this, will be populated at runtime
        }
    }
}


impl Tilemap {

    pub fn id(&self) -> u8 { self.id }


    pub fn serialize(&self) -> [u8; SIZE_OF_TILEMAP] {
        let mut bytes = ByteArray::<SIZE_OF_TILEMAP>::new();

        bytes.push(self.id);
        bytes.push(self.tileset);
        bytes.push(self.cols.to_ne_bytes()[0]);
        bytes.push(self.cols.to_ne_bytes()[1]);
        bytes.push(self.rows.to_ne_bytes()[0]);
        bytes.push(self.rows.to_ne_bytes()[1]);
        bytes.push(self.palette);

        for tile in self.tiles {
            let tile_data = tile.serialize();
            bytes.push_array(&tile_data);
        }

        bytes.validate_and_get_data()
    }


    pub fn deserialize(cursor:&mut Cursor<'_, u8>) -> Self {
        Self {
            id: cursor.advance(),
            tileset: cursor.advance(),
            cols: u16::from_ne_bytes([cursor.advance(), cursor.advance()]),
            rows: u16::from_ne_bytes([cursor.advance(), cursor.advance()]),
            palette: cursor.advance(),
            bg_buffers: Default::default(),
            tiles: core::array::from_fn(|_|{
                Tile::deserialize(cursor)
            })
        }

    }


    pub fn bg_buffer_count(&self) -> usize { self.bg_buffers.len() }


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


    fn get_index(&self, col:u16, row:u16) -> usize {
        #[cfg(debug_assertions)]
        if col >= self.cols || row >= self.rows {
            panic!("Invalid tilemap coordinates {}, {}", col, row)
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


    /// Checks if a tile on the given line coordinates is a collider, returns the collider tile and its coordinates, if any.
    pub fn collide_with_line(&self, x0:i32, y0:i32, x1:i32, y1:i32) -> Option<(Tile, Vec2<i32>)> { 

        if (x0, y0) == (x1, y1) { return  None }

        let w = x1 - x0;
        let h = y1 - y0;
        
        let longest = if w.abs() > h.abs() { w.abs() } else { h.abs() };
        let inc_x = w as f32 / longest as f32;
        let inc_y = h as f32 / longest as f32;

        let mut float_x = x0 as f32;
        let mut float_y = y0 as f32;

        for _ in 0 ..= longest as usize {
            float_x += inc_x;
            float_y += inc_y;
            let x = float_x.round() as i32;
            let y = float_y.round() as i32;
            if x < 0 || x >= self.cols as i32 { return None };
            if y < 0 || y >= self.rows as i32 { return None };
            if (x != x0) || (y != y0){
                let tile = self.get_tile(
                    u16::try_from(x).unwrap(),
                    u16::try_from(y).unwrap()
                );
                if tile.is_collider() {
                    // println!("collision {},{} -> {},{}", x0, y0, x1, y1);
                    // println!("{},{},{}; {},{}: {:?}", inc_x, inc_y, longest, x, y, tile);
                    return Some((tile, Vec2{x, y}))
                }  
            }
        }
        None
    }
    

    
}