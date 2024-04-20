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


    fn get_index(&self, col:u16, row:u16) -> Option<usize> {
        #[cfg(debug_assertions)]
        if col >= self.cols || row >= self.rows {
            // panic!("Invalid tilemap coordinates {}, {}", col, row)
            return None
        }
        Some((row as usize * self.cols as usize) + col as usize)
    }


    #[allow(unused)]
    pub fn set_tile(&mut self, col:u16, row:u16, value:Tile) {
        if let Some(index) = self.get_index(col, row) {
            self.tiles[index] = value;
        }
    }


    pub fn get_tile(&self, col:u16, row:u16) -> Option<Tile> {
        let index = self.get_index(col, row)?;
        Some(self.tiles[index])
    }


    pub fn width(&self, tile_width:u8) -> usize { self.cols as usize * tile_width as usize }


    pub fn height(&self, tile_height:u8) -> usize { self.rows as usize * tile_height as usize }


    /// Collision with ADJACENT TILES ONLY. If delta is more than one (single tile) in any axis result won't be correct.
    /// Function is long since it handles corner cases (literally corners!), but should still run pretty fast.
    pub fn collide_adjacent<T>(&self, x0:T, y0:T, x1:T, y1:T, filter:Axis) -> Option<Collision<T>>
    where T: Float {
        let y = y0.floor();
        let end_y = y1.floor();

        let x = x0.floor();
        let end_x = x1.floor();

        let start_col = x.to_u16()?;
        let start_row = y.to_u16()?;

        let end_col = end_x.to_u16()?;
        let end_row = end_y.to_u16()?;

        if start_col == end_col && start_row == end_row { return None }
        if start_col >= self.cols || end_col >= self.cols { return None }
        if start_row >= self.rows || end_row >= self.rows { return None }

        let dx = x1 - x0;
        let dy = y1 - y0;

        // let colliding_tile = || -> TileCollision {
        //     let col = end_col;
        //     let row = end_row;
        //     TileCollision {
        //         tile: self.get_tile(end_col, end_row),
        //         col, row
        //     }
        // };
        
        // Special case: Vertical movement only
        if start_col == end_col && filter != Axis::Horizontal {   
            let tile = self.get_tile(end_col, end_row)?;
            if tile.is_collider(){
                let dist_y = if dy.is_sign_positive() { y0.ceil() - y0 } else { y - y0 };
                return Some(Collision{
                    pos: Vec2 { x: x0, y: y0 + dist_y },
                    normal: Vec2{ x: T::zero(), y:-dy.signum() },
                    t: Vec2 { x:T::one(), y:dist_y.abs() / dy.abs()  },
                    tile: Some(TileCollision{ tile, col:end_col, row:end_row }),
                    velocity: Vec2::zero(),
                    colliding_entity: Default::default(),
                })
            }
        // Special case: Horizontal movement only
        } else if start_row == end_row && filter != Axis::Vertical {
            let dist_x = if dx.is_sign_positive() { x0.ceil() - x0 } else { x - x0 };
            let tile = self.get_tile(end_col, end_row)?;
            if tile.is_collider(){
                return Some(Collision{
                    pos: Vec2 { x: x0 + dist_x, y: y0 },
                    normal: Vec2{ x: -dx.signum(), y:T::zero() },
                    t: Vec2 { x:dist_x.abs() / dx.abs(), y:T::one()  },
                    tile: Some(TileCollision{ tile, col:end_col, row:end_row }),
                    velocity: Vec2::zero(),
                    colliding_entity: Default::default(),
                })
            }
        // Non Axis aligned movement
        } else {                
            // Account for concave corner on X
            let tile_x = self.get_tile(end_col, start_row)?;
            let (dist_x, normal_x) = if tile_x.is_collider() && filter != Axis::Vertical {(
                if dx.is_sign_positive() { x0.ceil() - x0 } else { x - x0 },
                -dx.signum()
            )} else {(
                T::one() * dx.signum(),
                T::zero()
            )};

            // Account for concave corner on Y
            let tile_y = self.get_tile(start_col, end_row)?;
            let (dist_y, normal_y) = if tile_y.is_collider() && filter != Axis::Horizontal {(
                if dy.is_sign_positive() { y0.ceil() - y0 } else { y - y0 },
                -dy.signum()
            )} else {(
                T::one() * dy.signum(),
                T::zero()
            )};

            // Return if either or both axes collided
            if tile_x.is_collider() || tile_y.is_collider() {
                let (tile, col, row) = if tile_x.is_collider() {(
                    tile_x, end_col, start_row
                )} else {(
                    tile_y, start_col, end_row
                )};
                return Some(Collision{
                    normal: Vec2 { x: normal_x, y: normal_y },
                    pos: Vec2 { x: x0 + dist_x, y: y0 + dist_y },
                    t: Vec2 { x:dist_x.abs() / dx.abs(), y:dist_y.abs() / dy.abs()  },
                    tile: Some( TileCollision{ tile, col, row } ),
                    colliding_entity: Default::default(),
                    velocity: Vec2::zero(),
                })
                
            // Otherwise check for a convex corner collision at the end tile coordinates (won't "slide")
            } else {    
                let tile = self.get_tile(end_col, end_row)?;
                if tile.is_collider(){
                    // To avoid getting stuck in a corner, we return a collision in one axis only
                    // May still cause hiccups in vertical gaps smaller than collider.
                    if dx.abs() < dy.abs() { 
                        if filter != Axis::Vertical {
                            let dist = if dx.is_sign_positive() { x0.ceil() - x0 } else { x - x0 };
                            return Some(Collision{
                                pos: Vec2 { x: x0 + dist, y: y1 },
                                normal: Vec2{ x: -dx.signum(), y: T::zero() },
                                t: Vec2 { x: dist.abs() / dx.abs(), y: T::one() },
                                tile: Some( TileCollision{ tile, col:end_col, row:end_row } ),
                                velocity: Vec2::zero(),
                                colliding_entity: Default::default(),
                            })
                        }
                    } else if filter != Axis::Horizontal {
                        let dist = if dy.is_sign_positive() { y0.ceil() - y0 } else { y - x0 };
                        return Some(Collision{
                            pos: Vec2 { x: x1, y: y0 + dist },
                            normal: Vec2{ x: T::zero() , y:-dy.signum() },
                            t: Vec2 { x: T::one(), y: dist.abs() / dy.abs() },
                            tile: Some( TileCollision{ tile, col:end_col, row:end_row } ),
                            velocity: Vec2::zero(),
                            colliding_entity: Default::default(),
                        })
                    }
                }
            }

        }

        None
    }


    /// Checks if a tile on the given line coordinates has its collider flag set to true
    pub fn raycast<T>(&self, x0:T, y0:T, x1:T, y1:T) -> Option<Collision<T>>
    where T: Float {

        let min = Vec2{
            x: x0.min(x1).floor().to_i32()?,
            y: y0.min(y1).floor().to_i32()?,
        };

        let max = Vec2{
            x: x0.max(x1).floor().to_i32()?,
            y: y0.max(y1).floor().to_i32()?,
        };

        // println!("\nRaycast: {:.2}, {:.2} -> {:.2}, {:.2}", x0, y0, x1, y1);
        if min.x == max.x && min.y == max.y { return None; }

        let mut coords = Vec2{
            x: x0.floor().to_i32()?,
            y: y0.floor().to_i32()?
        };

        let dx = x1 - x0;
        let dy = y1 - y0;

        let dir = Vec2 { x: dx, y: dy }.normalize();
        let start_point = Vec2{ x:x0, y:y0 };
        let end_point = Vec2{ x:x1, y:y1 };

        let step_mult = Vec2 {
            x: (T::one() + ((dir.y / dir.x) * (dir.y / dir.x))).sqrt(),
            y: (T::one() + ((dir.x / dir.y) * (dir.x / dir.y))).sqrt(),
        };

        let step = Vec2{
            x:if dir.x < T::zero() { -1 } else { 1 },
            y:if dir.y < T::zero() { -1 } else { 1 },
        };

        let mut ray_len = Vec2{
            // Initial values, will be mutated during the loop
            x: if dir.x > T::zero() {
                (T::from_i32(1 + coords.x)? - x0) * step_mult.x
            } else if dir.x < T::zero() {
                (x0 - T::from_i32(coords.x)?) * step_mult.x
            } else {
                T::max_value()
            },
            y: if dir.y > T::zero() {
                (T::from_i32(1 + coords.y)? - y0) * step_mult.y
            } else if dir.y < T::zero(){
                (y0 - T::from_i32(coords.y)?) * step_mult.y
            }else {
                T::max_value()
            }
        };

        let line_length = start_point.distance_to(end_point);

        let mut dist = T::zero();
        let mut normal:Vec2<T>;

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
            }

            if dist > line_length {  break }
            if coords.x > -1 && coords.x < self.cols as i32 && coords.y > -1 && coords.y < self.rows as i32 {
                let tile = self.get_tile(coords.x as u16, coords.y as u16)?;
                if tile.is_collider() {
                    // Calculate the precise intersection point
                    let edge_x = T::from(if dir.x < T::zero() { coords.x + 1 } else { coords.x })?;
                    let edge_y = T::from(if dir.y < T::zero() { coords.y + 1 } else { coords.y })?;

                    let pos = Vec2{
                        x: if dx == T::zero() {
                            x0
                        } else {
                            edge_x
                        },
                        y: if dy == T::zero() {
                            y0
                        } else {
                            edge_y
                        }
                    };
                    
                    let col = Collision {
                        pos,
                        normal,
                        t: Vec2 {
                            x: normal.y.abs(),
                            y:normal.x.abs()
                        },
                        tile: Some(TileCollision{
                            tile,
                            col: coords.x as u16,
                            row: coords.y as u16,
                        }),
                        // TODO: Will need to be populated by world!
                        velocity: Vec2::zero(),
                        colliding_entity: Default::default(),
                    };
                    // println!("{:?}", col);
                    return Some(col)
                }
            }
        }
        None
    }

}
