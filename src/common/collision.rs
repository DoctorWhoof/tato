use num_traits::Float;
use crate::*;

const COL_MARGIN:f32 = 0.01;

slotmap::new_key_type!{
    pub struct ColliderID;
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
    Stop,
    Bounce(f32),
    Slide
}


#[derive(Clone, Debug, Default)]
pub struct Collision<T> where T:Float + PartialOrd + Copy{
    pub tile:Option<Tile>,
    pub entity_id: EntityID,
    pub velocity:Vec2<T>,
    pub other_velocity:Vec2<T>,
    pub pre_col_delta:Vec2<T>,
    pub normal:Vec2<T>,
    pub t:T
}


#[derive(Clone, Debug, Default)]
pub struct IntermediateCollision<T> where T:Float + PartialOrd + Copy{
    pub pos:Vec2<T>,
    pub normal:Vec2<T>,
    pub t:T
}


/// Contains additional collider information, like velocity and start position
#[derive(Clone, Debug)]
pub struct CollisionProbe<T> {
    pub kind: ColliderKind,
    pub entity_id: EntityID,
    pub pos: Vec2<T>,
    pub velocity:Vec2<T>,
    pub layer: u8,
    pub mask: u8
}


#[derive(Clone, Copy, Debug)]
pub struct Collider{
    pub kind: ColliderKind,
    pub pos: Vec2<f32>,
    pub enabled: bool,
    pub layer: u8,
    pub mask: u8
}


#[derive(Debug)]
pub struct Ray<T> {
    pub origin: Vec2<T>,
    pub angle: T, // In radians
}



impl Collider {

    // TODO: verify that pos, w and h are overwritten by world when adding the collision probe - even when the map moves!
    // No tilemap reference used! The collider's values are set on the fly when the collider is added
    pub fn new_tilemap_collider() -> Self {
        Self {
            enabled: true,
            pos: Vec2::zero(),  
            kind: ColliderKind::Tilemap { w:0.0, h:0.0, tile_width:0, tile_height:0 },
            layer: 0,
            mask: 0,
        }
    }

    pub fn new_point_collider(x:f32, y:f32) -> Self {
        Self {
            enabled: true,
            pos: Vec2 { x, y },
            kind: ColliderKind::Point,
            layer: 0,
            mask: 0,
        }
    }

    pub fn new_rect_collider(x:f32, y:f32, w:f32, h:f32) -> Self {
        Self {
            enabled: true,
            pos: Vec2 { x, y },
            kind: ColliderKind::Rect{ w, h },
            layer: 0,
            mask: 0,
        }
    }
    
}


impl CollisionProbe<f32> {

    fn collision_from_intermediate(&self, maybe_value: Option<IntermediateCollision<f32>>) -> Option<Collision<f32>> {
        let value = maybe_value?;
        Some(Collision {
            tile: None,
            entity_id: Default::default(),
            normal: value.normal,
            t: value.t,
            // will be filled later. TODO: I don't like this
            velocity: Vec2::zero(),
            other_velocity: Vec2::zero(),
            pre_col_delta: Vec2::zero(),  
        })
    }


    pub fn collision_response(&self, other:&Self, tilemap:Option<&Tilemap>) -> Option<Collision<f32>> {
        
        if !self.broad_phase_overlaps(other) { return None }
                
        if let Some(col) = self.refine_collision(other, tilemap) {
            let mut col = col;
            // Safety check, interpolation outside the unit range means no collision!
            // Seems to help preventing glitches?
            if col.t < 0.0 || col.t > 1.0 { return None };
            // Calculate margin
            let margin_x = (COL_MARGIN * col.normal.x) + other.velocity.x;
            let margin_y = (COL_MARGIN * col.normal.y) + other.velocity.y;
            // "Stop" distance delta
            col.pre_col_delta.x = (self.velocity.x * col.t) + margin_x;
            col.pre_col_delta.y = (self.velocity.y * col.t) + margin_y;
            // Turns the incoming collider velocity into additional self velocity
            col.velocity = Vec2 {
                x: self.velocity.x - other.velocity.x ,
                y: self.velocity.y - other.velocity.y
            };
            col.other_velocity = other.velocity;
            return Some(col);
        }

        None
    }


    // Performs collision checks using raycasts to obtain a collision normal and its location.
    // Assume broad AABB collision has already happened!
    fn refine_collision(&self, other:&Self, tilemap:Option<&Tilemap>) -> Option<Collision<f32>> {
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
                        let line = Line{start:self.pos, end:self.pos + self.velocity - other.velocity};
                        self.collision_from_intermediate( other_rect.intersect_line(&line) )
                    },
                    // Rect to Rect
                    ColliderKind::Rect{ .. } => {
                        let rect = Rect::from(self);
                        Self::sweep_rect_to_rect_colllision(rect, other_rect, self.velocity)
                    },
                    // Tilemap to Rect
                    ColliderKind::Tilemap { .. } => None,
                }
            },
            // Point to tilemap
            ColliderKind::Tilemap{ tile_width, tile_height, .. } => {
                let tilemap = tilemap?;
                let tilemap_rect = Rect::from(other);
                match self.kind {
                    // Point to Tilemap
                    ColliderKind::Point => {
                        let x0 = self.pos.x - tilemap_rect.x;
                        let y0 = self.pos.y - tilemap_rect.y;
                        let x1 = x0 + self.velocity.x;
                        let y1 = y0 + self.velocity.y;
                        self.collision_from_intermediate( tilemap.raycast(x0, y0, x1, y1) )
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
                                let x0 = x + self.pos.x - tilemap_rect.x;
                                let y0 = y + self.pos.y - tilemap_rect.y;
                                let x1 = x0 + self.velocity.x;
                                let y1 = y0 + self.velocity.y;
                                if let Some(col) = self.collision_from_intermediate( tilemap.raycast(x0, y0, x1, y1) ){
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


    // fn sweep_rect_to_rect_colllision(a:Rect<f32>, b:Rect<f32>, vel_a:Vec2<f32>, vel_b:Vec2<f32>) -> Option<Collision<f32>> {
    fn sweep_rect_to_rect_colllision(a:Rect<f32>, b:Rect<f32>, vel_a:Vec2<f32>) -> Option<Collision<f32>> {
        // find the distance between the objects on the near and far sides for both x and y 
        let (dist_entry_x, dist_exit_x) = if vel_a.x > 0.0 { 
            (b.x - a.right(), b.right() - a.x)
        } else { 
            (b.right() - a.x, b.x - a.right())
        };

        let (dist_entry_y, dist_exit_y) = if vel_a.y > 0.0 { 
            (b.y - a.bottom(),  b.bottom() - a.y)
        } else { 
            (b.bottom() - a.y, b.y - a.bottom())
        };

        let (entry_x, exit_x) = if vel_a.x == 0.0 { 
            (f32::NEG_INFINITY, f32::INFINITY)
        } else { 
            (dist_entry_x / vel_a.x, dist_exit_x / vel_a.x)
        };

        let (entry_y, exit_y) = if vel_a.y == 0.0 { 
            (f32::NEG_INFINITY, f32::INFINITY)
        } else { 
            (dist_entry_y / vel_a.y, dist_exit_y / vel_a.y)
        };

        let entry_time = entry_x.max(entry_y); 
        let exit_time = exit_x.min(exit_y);

        if (entry_time > exit_time) || (entry_x < 0.0 && entry_y < 0.0) || (entry_x > 1.0 || entry_y > 1.0) { 
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
        Some(Collision{
            tile: None,
            entity_id: Default::default(),
            velocity: vel_a,
            other_velocity: Vec2::zero(),   //vel_b
            pre_col_delta: Vec2{
                x: vel_a.x * entry_time,
                y: vel_a.y * entry_time,
            },
            normal,
            t: entry_time
        })
    }



    // fn line_in_rect_collision(&self, rect:Rect<f32>) -> Option<IntermediateCollision<f32>> {
    //     let trajectory = Ray { origin: self.pos, angle: self.velocity.y.atan2(self.velocity.x) + PI };
    //     if let Some(mut col) = rect.intersect_line(&trajectory){
    //         // TODO: This seems slower than necessary?
    //         // Maybe interset_ray can return the correct interpolation amount without further calculation?
    //         let len = self.velocity.len();
    //         if len > 0.0 {
    //             // println!("start:{:.2?}, end:{:.2?}", self.start_position, col.pos);
    //             let dist = self.pos.distance_to(col.pos).abs();
    //             // println!("distance:{:.02?}", dist);
    //             col.t =  dist / len;
    //         }

    //         return Some(col)
    //     }
    //     None
    // }


    fn broad_phase_overlaps(&self, other:&Self) -> bool {
        match self.kind {
            ColliderKind::Point => {
                match other.kind {
                    // Point in point
                    ColliderKind::Point => self.pos.floor() == other.pos.floor(),
                    // Point in rect
                    ColliderKind::Rect{ .. } | ColliderKind::Tilemap { .. }=> {
                        let rect = Rect::from(other);
                        Self::broad_phase_point_in_rect(self.pos, self.velocity, rect, other.velocity)
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
                    ColliderKind::Point => rect.contains(other.pos.x, other.pos.y),
                    // Rect over Rect
                    ColliderKind::Rect{ .. } | ColliderKind::Tilemap { .. }=> {
                        let other_rect = Rect::from(other);
                        Self::broad_phase_rects_overlap(rect, other_rect, self.velocity)
                    },
                }
            },
            ColliderKind::Tilemap { .. } => {
                let rect = Rect::from(self);
                match other.kind {
                    // Rect over point
                    ColliderKind::Point => rect.contains(other.pos.x, other.pos.y),
                    // Rect over rect
                    ColliderKind::Rect{ .. } | ColliderKind::Tilemap { .. }=> {
                        let other_rect = Rect::from(other);
                        rect.overlaps(&other_rect)
                    },
                }
            }
        }
    }



    pub fn broad_rect(a:Rect<f32>, vel:Vec2<f32>) -> Rect<f32> {
        Rect{
            x: if vel.x > 0.0 { a.x } else { a.x + vel.x },
            y: if vel.y > 0.0 { a.y } else { a.y + vel.y },
            w: if vel.x > 0.0 { vel.x + a.w } else { a.w - vel.x },
            h: if vel.y > 0.0 { vel.y + a.h } else { a.h - vel.y },
        }
    }


    pub fn broad_phase_point_in_rect(point: Vec2<f32>, point_vel: Vec2<f32>, rect: Rect<f32>, rect_vel:Vec2<f32>) -> bool {
        // rect sweeping method, needs more testing! May fail at high speeds, AABB sweep may be too broad?
        let broad_rect = Self::broad_rect(rect, rect_vel + point_vel.scale(-1.0));
        broad_rect.contains(point.x, point.y)|| broad_rect.contains(point.x + point_vel.x, point.y + point_vel.y)
    }


    // TODO: needs vel_b
    pub fn broad_phase_rects_overlap(a:Rect<f32>, b:Rect<f32>, vel_a:Vec2<f32>) -> bool {
        let broad_rect = Self::broad_rect(a, vel_a);
        broad_rect.overlaps(&b)
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


impl From<Vec2<f32>> for Collider {
    fn from(value: Vec2<f32>) -> Self {
        Self {
            enabled: true,
            pos: value,
            kind: ColliderKind::Point,
            layer: 0,
            mask: 0,
        }
    }
}


impl From<Rect<f32>> for Collider {
    fn from(value: Rect<f32>) -> Self {
        Self {
            enabled: true,
            pos: Vec2 { x:value.x, y:value.y },
            kind: ColliderKind::Rect{ w:value.w, h:value.h },
            layer: 0,
            mask: 0,
        }
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