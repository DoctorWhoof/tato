use core::ops::{Add,AddAssign};
use num_traits::Float;
use crate::*;

const COL_MARGIN:f32 = 0.2;


#[derive(Clone, Copy, Debug)]#[repr(u8)]
enum Axis {
    Horizontal,
    Vertical
}


#[derive(Clone, Copy, Debug)]#[repr(u8)]
pub enum ColliderKind{
    Point,
    Rect{w:f32, h:f32},
    Tilemap{w:f32, h:f32, tile_width:u8, tile_height:u8} // Is populated by World, values depend on the tilemap
}


#[derive(Debug, Clone, Copy)]
pub enum CollisionReaction {
    None,
    Bounce(f32),
    Slide
}


/// Contains details about a collision that occurred in the current frame.
#[derive(Clone, Debug)]
pub struct Collision<T> where T:Float + PartialOrd + Copy{
    // pub tile:Option<Tile>,
    // pub pos: Coords,
    pub tile_coords:Option<Vec2<i32>>,
    pub entity_id: EntityID,
    pub velocity:Vec2<T>,
    pub margin:Vec2<T>,
    pub normal:Vec2<T>,
}


#[derive(Clone, Debug, Default)]
pub(crate) struct AxisCollision<T> where T:Float + PartialOrd + Copy{
    pub pos:Coords,
    // pub tile_coord:Option<i32>,
    pub velocity:T,
    pub margin:T,
    pub normal:T,
    pub t:T,
}


#[derive(Clone, Debug)]
pub(crate) struct IntermediateCollision<T> where T:Float + PartialOrd + Copy{
    pub pos: Coords,
    pub normal:Vec2<T>,
    pub t:T
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum Coords {
    #[default]
    None,
    Tile { col:i32, row:i32 },
    World { x:f32, y:f32 },
    Horizontal(f32),
    Vertical(f32)
}

/// Generated when checking for collisions, contains additional collider information like velocity and start position.
#[derive(Clone, Debug)]
pub struct CollisionProbe<T> {
    pub kind: ColliderKind,
    pub pos: Vec2<T>,
    pub entity_id: EntityID,
    pub velocity:Vec2<T>,
    pub layer: u16,
    pub mask: u16
}

/// Allows an entity to specify a collision shape, a position offset and collision layer masking.
#[derive(Clone, Copy, Debug)]
pub struct Collider{
    pub kind: ColliderKind,
    pub pos: Vec2<f32>,
    pub enabled: bool,
    pub mask: u16,
    pub(crate) layer: u16,
}



impl Collider {

    pub fn new_tilemap_collider(layer:impl CollisionLayer) -> Self {
        Self {
            enabled: true,
            pos: Vec2::zero(),
            kind: ColliderKind::Tilemap { w:0.0, h:0.0, tile_width:0, tile_height:0 }, // Values will be written by World
            layer: layer.into(),
            mask: 0,
        }
    }

    pub fn new_point_collider(layer:impl CollisionLayer, x:f32, y:f32) -> Self {
        Self {
            enabled: true,
            pos: Vec2 { x, y },
            kind: ColliderKind::Point,
            layer: layer.into(),
            mask: 0,
        }
    }

    pub fn new_rect_collider(layer:impl CollisionLayer, rect:Rect<f32>) -> Self {
        Self {
            enabled: true,
            pos: Vec2 { x:rect.x, y:rect.y },
            kind: ColliderKind::Rect{ w:rect.w, h:rect.h },
            layer: layer.into(),
            mask: 0,
        }
    }

}


impl CollisionProbe<f32> {

    pub(crate) fn collision_response(&self, other:&Self, tilemap:Option<&Tilemap>) -> (Option<AxisCollision<f32>>, Option<AxisCollision<f32>>) {

        let mut result = (None, None);

        // X Step
        if self.broad_phase(other, Axis::Horizontal) {
            if let Some(col) = self.axis_collision(other, Axis::Horizontal, tilemap) {
                let mut col_x = col;
                // Expand margin on movement. Helps with stability
                col_x.margin += other.velocity.x;
                // Turns the incoming collider velocity into additional self velocity
                col_x.velocity = self.velocity.x - other.velocity.x;
                result.0 = Some(col_x);
            } else {
                // println!("uh oh, broad collision but not axis collision in X!")
            }
        }

        // Y Step
        if self.broad_phase(other, Axis::Vertical) {
            if let Some(col) = self.axis_collision(other, Axis::Vertical, tilemap) {
                let mut col_y = col;
                // Expand margin on movement. Helps with stability
                col_y.margin += other.velocity.y;
                // Turns the incoming collider velocity into additional self velocity
                col_y.velocity = self.velocity.y - other.velocity.y;
                result.1 = Some(col_y);
            } else {
                // println!("uh oh, broad collision but not axis collision in Y!")
            }
        }

        result
    }


    // Performs collision checks using raycasts to obtain a collision normal and its location.
    // Assume broad AABB collision has already happened!
    fn axis_collision(&self, other:&Self, axis:Axis, tilemap:Option<&Tilemap>) -> Option<AxisCollision<f32>> {
        let (vel_self, vel_other) = match axis {
            Axis::Horizontal => (self.velocity.horiz(), other.velocity.horiz()),
            Axis::Vertical => (self.velocity.vert(), other.velocity.vert()),
        };
        match other.kind {
            ColliderKind::Point => {
                match self.kind {
                    // Point to Point
                    ColliderKind::Point => None,
                    // Rect to Point
                    ColliderKind::Rect{ .. } => None,
                    // Tilemap to Point
                    ColliderKind::Tilemap { .. } => None,
                }
            },
            // Point to Rect
            ColliderKind::Rect{..} => {
                let other_rect = Rect::from(other);
                // let other_rect = Self::broad_rect(other_rect, other.velocity);
                match self.kind {
                    // Point to Rect
                    ColliderKind::Point => {
                        self.axis_col_from_intermediate(
                            match axis {
                                Axis::Horizontal => Self::sweep_point_in_rect_x(self.pos.x, vel_self.x, other_rect, vel_other.x),
                                Axis::Vertical => Self::sweep_point_in_rect_y(self.pos.y, vel_self.y, other_rect, vel_other.y),
                            },
                            axis
                        )
                    },
                    // Rect to Rect
                    ColliderKind::Rect{ .. } => {
                        let rect = Rect::from(self);
                        self.axis_col_from_intermediate(
                            Self::sweep_rect_to_rect_colllision(rect, other_rect, vel_self, vel_other)
                            , axis
                        )
                    },
                    // Tilemap to Rect
                    ColliderKind::Tilemap { .. } => None,
                }
            },
            // Point to tilemap
            ColliderKind::Tilemap{ tile_width, tile_height, .. } => {
                let tilemap = tilemap?;
                let tilemap_rect = Rect::from(other);

                // Scaling - needs testing!
                let pos_x = self.pos.x / tile_width as f32;
                let pos_y = self.pos.y / tile_height as f32;
                let vel_x = vel_self.x / tile_width as f32;
                let vel_y = vel_self.y / tile_height as f32;

                match self.kind {
                    // Point to Tilemap
                    ColliderKind::Point => {
                        let x0 = pos_x - tilemap_rect.x;
                        let y0 = pos_y - tilemap_rect.y;
                        let x1 = x0 + vel_x;
                        let y1 = y0 + vel_y;
                        self.axis_col_from_intermediate( tilemap.raycast(x0, y0, x1, y1), axis )
                    },
                    // Rect to Tilemap
                    ColliderKind::Rect{ w, h } => {
                        let tiles_h = (w / tile_width as f32).floor() as usize;
                        let tiles_v = (h / tile_height as f32).floor() as usize;
                        // println!("{}, {}", tiles_h, tiles_v);
                        for y in 0 ..= tiles_v {
                            let y = y as f32;
                            for x in 0 ..= tiles_h {
                                let x = x as f32;
                                let x0 = x + pos_x - tilemap_rect.x;
                                let y0 = y + pos_y - tilemap_rect.y;
                                let x1 = x0 + vel_x;
                                let y1 = y0 + vel_y;
                                if let Some(col) = self.axis_col_from_intermediate( tilemap.raycast(x0, y0, x1, y1), axis ){
                                    return Some(col)
                                }
                            }
                        }
                        None
                    },
                    // Tilemap to Tilemap
                    ColliderKind::Tilemap { .. } => None,
                }
            },
        }
    }


    fn broad_phase(&self, other:&Self, axis:Axis) -> bool {
        let (vel_self, vel_other) = match axis {
            Axis::Horizontal => (self.velocity.horiz(), other.velocity.horiz()),
            Axis::Vertical => (self.velocity.vert(), other.velocity.vert()),
        };
        match self.kind {
            ColliderKind::Point => {
                match other.kind {
                    // Point in point
                    ColliderKind::Point => false,
                    // Point in rect / Point in whole tilemap rect
                    ColliderKind::Rect{ .. } | ColliderKind::Tilemap { .. }=> {
                        let other_rect = Rect::from(other);
                        Self::broad_phase_point_in_rect(self.pos, vel_self, other_rect, vel_other)
                    },
                }
            },
            ColliderKind::Rect{ w, h } => {
                let rect = Rect {
                    x: self.pos.x,
                    y: self.pos.y,
                    w, h
                };
                match other.kind {
                    // Rect over point
                    ColliderKind::Point => {
                        Self::broad_phase_point_in_rect(other.pos, vel_other, rect, vel_self)
                    }
                    // Rect over Rect, including tilemap rect
                    ColliderKind::Rect{ .. } | ColliderKind::Tilemap { .. }=> {
                        let other_rect = Rect::from(other);
                        Self::broad_phase_rects_overlap(rect, other_rect, vel_self, vel_other)
                    },
                }
            },
            ColliderKind::Tilemap { .. } => {
                // Collisions can't originate from tilemaps
                false
            }
        }
    }



    pub(crate) fn broad_rect(a:Rect<f32>, vel:Vec2<f32>) -> Rect<f32> {
        Rect{
            x: if vel.x > 0.0 { a.x } else { a.x + vel.x },
            y: if vel.y > 0.0 { a.y } else { a.y + vel.y },
            w: if vel.x > 0.0 { vel.x + a.w } else { a.w - vel.x },
            h: if vel.y > 0.0 { vel.y + a.h } else { a.h - vel.y },
        }
    }


    // rect sweeping method, needs more testing!
    pub(crate) fn broad_phase_point_in_rect(point: Vec2<f32>, point_vel: Vec2<f32>, rect: Rect<f32>, rect_vel:Vec2<f32>) -> bool {
        let broad_rect = Self::broad_rect(rect, rect_vel - point_vel);
        broad_rect.contains(point.x, point.y)
    }


    pub(crate) fn broad_phase_rects_overlap(a:Rect<f32>, b:Rect<f32>, vel_a:Vec2<f32>, vel_b:Vec2<f32>) -> bool {
        let broad_rect_a = Self::broad_rect(a, vel_a);
        let broad_rect_b = Self::broad_rect(b, vel_b);
        broad_rect_a.overlaps(&broad_rect_b)
    }


    pub(crate) fn sweep_point_in_rect_x(x:f32, vel_x:f32, rect:Rect<f32>, rect_vel_x:f32) ->  Option<IntermediateCollision<f32>> {
        let rect_left = if rect_vel_x > 0.0 { rect.x } else { rect.x + rect_vel_x };
        let rect_right = if rect_vel_x > 0.0 { rect.right() + rect_vel_x } else { rect.right() };
        let result_vel = vel_x - rect_vel_x;
        if result_vel == 0.0 {
            None
        } else if result_vel > 0.0 {
            Some(IntermediateCollision{
                pos: Coords::Horizontal(rect_left),
                normal: Vec2::left(),
                t:(rect_left - x) / result_vel
            })
        } else if result_vel < 0.0 {
            Some(IntermediateCollision{
                pos: Coords::Horizontal(rect_right),
                normal: Vec2::right(),
                t:(rect_right - x) / result_vel
            })
        } else {
            None
        }
    }


    pub(crate) fn sweep_point_in_rect_y(y:f32, vel_y:f32, rect:Rect<f32>, rect_vel_y:f32) ->  Option<IntermediateCollision<f32>> {
        let rect_top = if rect_vel_y > 0.0 { rect.y } else { rect.y + rect_vel_y };
        let rect_bottom = if rect_vel_y > 0.0 { rect.bottom() + rect_vel_y } else { rect.bottom() };
        let result_vel = vel_y - rect_vel_y;
        if result_vel == 0.0 {
            None
        } else if result_vel > 0.0 {
            Some(IntermediateCollision{
                pos: Coords::Vertical(rect_top),
                normal: Vec2::up(),
                t:(rect_top - y) / result_vel
            })
        } else if result_vel < 0.0 {
            Some(IntermediateCollision{
                pos: Coords::Vertical(rect_bottom),
                normal: Vec2::down(),
                t:(rect_bottom - y) / result_vel
            })
        } else {
            None
        }
    }


    fn sweep_rect_to_rect_colllision(a:Rect<f32>, b:Rect<f32>, vel_a:Vec2<f32>, vel_b:Vec2<f32>) -> Option<IntermediateCollision<f32>> {
        // find the distance between the objects on the near and far sides for both x and y
        let vel = vel_a - vel_b;

        let dist_entry_x = if vel.x > 0.0 {
            b.x - a.right()
        } else {
            b.right() - a.x
        };

        let dist_entry_y = if vel.y > 0.0 {
            b.y - a.bottom()
        } else {
            b.bottom() - a.y
        };

        let entry_x = if vel.x == 0.0 {
            f32::NEG_INFINITY
        } else {
            dist_entry_x / vel.x
        };

        let entry_y = if vel.y == 0.0 {
            f32::NEG_INFINITY
        } else {
            dist_entry_y / vel.y
        };

        let entry_time = entry_x.max(entry_y);

        let safety = 16.0;   //Hack! TODO: Make more sense of this...
        if (entry_x < -safety && entry_y < -safety) || (entry_x > safety || entry_y > safety)  {
            return None
        }

        let normal:Vec2<f32> = if entry_x > entry_y {
            if dist_entry_x < 0.0 {
                Vec2::right()
            } else {
                Vec2::left()
            }
        } else if dist_entry_y < 0.0 {
            Vec2::down()
        } else {
            Vec2::up()
        };

        // println!("normal:{:.2?}, entry_time:{:.2}, dist_x:{:.1}, entry_x:{:.1}", normal, entry_time, dist_entry_x, entry_x);
        Some(IntermediateCollision{
            pos: Coords::World { x: a.x + dist_entry_x, y: a.y + dist_entry_y }, //TODO: Needs testing
            normal,
            t: entry_time
        })
    }


    fn axis_col_from_intermediate(&self, maybe_value: Option<IntermediateCollision<f32>>, axis: Axis) -> Option<AxisCollision<f32>> {
        let value = maybe_value?;
        Some(AxisCollision {
            pos: value.pos,
            normal: match axis {
                Axis::Horizontal => value.normal.x,
                Axis::Vertical => value.normal.y,
            },
            t: value.t,
            margin: match axis {
                Axis::Horizontal => COL_MARGIN * value.normal.x,
                Axis::Vertical => COL_MARGIN * value.normal.y
            },
            velocity: 0.0,
        })
    }

}



impl From<&CollisionProbe<f32>> for Rect<f32> {
    fn from(col: &CollisionProbe<f32>) -> Self {
        let (w,h) = match col.kind {
            ColliderKind::Point => (1.0, 1.0),
            ColliderKind::Rect { w, h } => (w,h),
            ColliderKind::Tilemap { w, h, .. } => (w, h)
        };
        Rect{x: col.pos.x, y: col.pos.y, w, h}
    }
}


// impl From<Vec2<f32>> for Collider {
//     fn from(value: Vec2<f32>) -> Self {
//         Self {
//             enabled: true,
//             pos: value,
//             kind: ColliderKind::Point,
//             layer: 0,
//             mask: 0,
//         }
//     }
// }


// impl From<Rect<f32>> for Collider {
//     fn from(value: Rect<f32>) -> Self {
//         Self {
//             enabled: true,
//             pos: Vec2 { x:value.x, y:value.y },
//             kind: ColliderKind::Rect{ w:value.w, h:value.h },
//             layer: 0,
//             mask: 0,
//         }
//     }
// }


impl<T> AddAssign<AxisCollision<T>> for AxisCollision<T>
where T:Add<Output = T> + AddAssign + Copy + PartialOrd + Float {
    fn add_assign(&mut self, other: AxisCollision<T>) {
        self.velocity += other.velocity;
        self.normal += other.normal;
        self.margin += other.margin;
        self.t = self.t.min(other.t);   //Smallest is best?
    }
}




// let mut secondary_probe = self.clone();
// secondary_probe.velocity = result_velocity
// let mut secondary_probe = CollisionProbe{
//     entity_id: self.entity_id,
//     collider: self.collider,
//     start_position: col.pos,
//     velocity: col.velocity,
// };
// secondary_probe.collider.pos = col.pos + (col.velocity.scale(_elapsed));
// if let Some(_new_col) = secondary_probe.refine_collision(&other.collider, tilemap) {
//     println!("Secondary collision");
//     // col.pos = self.start_position;
//     // return Some(Collision{
//     //     tile: None,
//     //     entity_id: self.entity_id,
//     //     velocity: Vec2::reflect(col.velocity, col.normal).scale(bounce),
//     //     point: self.start_position,
//     //     normal: new_col.normal,
//     // })
// }



// pub fn horizontal_cast<T>(&self, x0:T, x1:T, y:T) -> Option<Collision<T>>
// where T: Float {
//     let mut x = x0.floor();
//     let end_x = x1.floor();
//     if x == end_x { return None }

//     let dx = x1 - x0;
//     let step = dx.signum();
//     let mut dist = if step > T::zero() { x0.ceil() - x0 } else { x0 - x };

//     // println!("{} -> {}, step:{}, dist:{}", x, end_x, step, dist);
//     loop {
//         x += step;
//         if step > T::zero() && x > end_x { break }
//         if step < T::zero() && x < end_x { break }

//         let col = x.to_u16()?;
//         let row = y.to_u16()?;
        
//         if col >= self.cols { break }

//         let tile = self.get_tile(col, row);
//         if tile.is_collider(){
//             return Some(Collision{
//                 t: Vec2 { x: dist / dx.abs(), y: T::one() },
//                 pos: Vec2 { x: x0 + dist, y },
//                 normal: Vec2 { x: -step, y: T::zero() },
//                 tile: Some(TileCollision{ tile, row, col }),
//                 velocity: Vec2::zero(),
//                 colliding_entity: Default::default(),
//             })
//         }

//         dist += step.abs();
//     }
//     None
// }



// pub fn vertical_cast<T>(&self, y0:T, y1:T, x:T) -> Option<Collision<T>>
// where T: Float {
//     let mut y = y0.floor();
//     let end_y = y1.floor();
//     if y == end_y { return None }

//     let dy = y1 - y0;
//     let step = dy.signum();
//     let mut dist = if step > T::zero() { y0.ceil() - y0 } else { y0 - y };

//     // println!("{} -> {}, step:{}, dist:{}", y, end_y, step, dist);
//     loop {
//         y += step;
//         if step > T::zero() && y > end_y { break }
//         if step < T::zero() && y < end_y { break }

//         let col = x.to_u16()?;
//         let row = y.to_u16()?;
        
//         if col >= self.cols { break }

//         let tile = self.get_tile(col, row);
//         if tile.is_collider(){
//             return Some(Collision{
//                 t: Vec2 { x: T::one(), y: dist / dy.abs() },
//                 pos: Vec2 { x, y: y0 + dist },
//                 normal: Vec2 { x: T::zero(), y: -step },
//                 tile: Some(TileCollision{ tile, row, col }),
//                 velocity: Vec2::zero(),
//                 colliding_entity: Default::default(),
//             })
//         }

//         dist += step.abs();
//     }
//     None
// }
