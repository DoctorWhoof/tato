use core::mem::size_of;
use slotmap::SecondaryMap;
use crate::*;

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
            }),
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


    /// Checks if a tile on the given line coordinates has its collider flag set to true
    pub(crate) fn raycast<T:Into<f64>>(&self, x0:T, y0:T, x1:T, y1:T) -> Option<IntermediateCollision<f32>> {
        let x0:f64 = x0.into();
        let x1:f64 = x1.into();
        let y0:f64 = y0.into();
        let y1:f64 = y1.into();

        let min = Vec2{
            x: x0.min(x1).floor() as i32,
            y: y0.min(y1).floor() as i32,
        };

        let max = Vec2{
            x: x0.max(x1).floor() as i32,
            y: y0.max(y1).floor() as i32,
        };

        // println!("\nRaycast: {:.2}, {:.2} -> {:.2}, {:.2}", x0, y0, x1, y1);
        if min.x == max.x && min.y == max.y {
            return None;
        }

        let mut coords = Vec2{
            x: x0.floor() as i32,
            y: y0.floor() as i32
        };


        let dir = Vec2 { x: x1 - x0, y: y1 - y0 }.normalize();
        let start_point = Vec2{ x:x0, y:y0 };
        let end_point = Vec2{ x:x1, y:y1 };


        let step_mult = Vec2 {
            x: (1.0 + ((dir.y / dir.x) * (dir.y / dir.x))).sqrt(),
            y: (1.0 + ((dir.x / dir.y) * (dir.x / dir.y))).sqrt(),
        };

        let step = Vec2{
            x:if dir.x < 0.0 { -1 } else { 1 },
            y:if dir.y < 0.0 { -1 } else { 1 },
        };

        let mut ray_len = Vec2{
            // Initial values, will be mutated during the loop
            x: if dir.x > 0.0 {
                ((1 + coords.x) as f64 - x0) * step_mult.x
            } else if dir.x < 0.0 {
                (x0 - coords.x as f64) * step_mult.x
            } else {
                f64::MAX
            },
            y: if dir.y > 0.0 {
                ((1 + coords.y) as f64 - y0) * step_mult.y
            } else if dir.y < 0.0{
                (y0 - coords.y as f64) * step_mult.y
            }else {
                f64::MAX
            }
        };

        let line_length = start_point.distance_to(end_point);
        let mut dist = 0.0;
        let mut normal:Vec2<f32>;

        // println!("\nRaycast: {:.2}, {:.2} -> {:.2}, {:.2}, coords:{:?}", x0, y0, x1, y1, coords);
        while dist < line_length { // TODO: use max_dist as a function parameter, get the "min" from it and line length
            if ray_len.x < ray_len.y {
                coords.x += step.x;
                dist = ray_len.x;
                ray_len.x += step_mult.x;
                if step.x < 0 {
                    if coords.x < min.x { break }
                    normal = Vec2::right();
                } else {
                    if coords.x > max.x { break }
                    normal = Vec2::left();
                };
                // println!("step X, ray_len.x:{:.2}, dist:{:.2}, coords:{:?}", ray_len.x, dist, coords);
            } else {
                coords.y += step.y;
                dist = ray_len.y;
                ray_len.y += step_mult.y;
                if step.y < 0 {
                    if coords.y < min.y { break }
                    normal = Vec2::down();
                } else {
                    if coords.y > max.y { break }
                    normal = Vec2::up();
                };
                // println!("step Y, ray_len.y:{:.2}, dist:{:.2}, coords:{:?}", ray_len.y, dist, coords);
            }

            if dist > line_length {  break }
            if coords.x > -1 && coords.x < self.cols as i32 && coords.y > -1 && coords.y < self.rows as i32 {
                let tile = self.get_tile(coords.x as u16, coords.y as u16);
                if tile.is_collider() {
                    // let intersection = Vec2 {
                    //     x: (x0 + (dir.x * dist)) as f32,
                    //     y: (y0 + (dir.y * dist)) as f32,
                    // };
                    let col = IntermediateCollision{
                        // velocity: Vec2::zero(), // Will be filled by the caller
                        // pos: intersection,
                        normal,
                        t: 0.0
                    };
                    // println!("Collision at {:?}, normal:{}", coords, normal);
                    return Some(col)
                }
            }
        }
        None
    }

}
