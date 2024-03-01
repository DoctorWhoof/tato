use core::f32::consts::PI;
use num_traits::Float;
use crate::*;

const COL_MARGIN:f32 = 0.1;

slotmap::new_key_type!{
    pub struct ColliderID;
}


#[derive(Clone, Copy, Debug)]#[repr(u8)]
pub enum ColliderKind{
    Point,
    Rect{w:f32, h:f32},
    Tilemap{w:f32, h:f32} // Is populated by World, values depend on the tilemap
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
    pub pos:Vec2<T>,
    pub normal:Vec2<T>,
    pub interp_amount:T
}


/// Contains additional collider information, like velocity and start position
#[derive(Clone, Debug)]
pub struct CollisionProbe<T> {
    pub collider:Collider,  // Contains the world space collider (obtained with entity.world_collider())
    pub entity_id: EntityID,
    pub start_position: Vec2<T>,
    pub velocity:Vec2<T>,
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
            kind: ColliderKind::Tilemap { w:0.0, h:0.0 },
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

    fn start_rect(&self) -> Rect<f32> {
        let (w,h) = match self.collider.kind {
            ColliderKind::Point => (1.0 ,1.0),
            ColliderKind::Rect { w, h } | ColliderKind::Tilemap { w, h, .. } => (w, h)
        };
        Rect {
            x: self.start_position.x,
            y: self.start_position.y,
            w, h
        }
    }


    pub fn collision_response(&self, other:&Self, reaction:CollisionReaction, tilemap:Option<&Tilemap>) -> Option<Collision<f32>> {
        
        if !self.broad_phase_overlaps(other) { return None }

        // Turns the incoming collider velocity into additional self velocity
        // let result_velocity = Vec2::weighted_add(self.velocity, other.velocity, 1.0, -1.0);
        let result_velocity = Vec2 {
            x: self.velocity.x - other.velocity.x ,
            y: self.velocity.y - other.velocity.y
        };

        let mut probe = self.clone();
        probe.velocity = result_velocity;
        if let Some(mut col) = probe.refine_collision(&other.collider, tilemap) {
            // Apply margin
            col.pos.x += COL_MARGIN * col.normal.x;
            col.pos.y += COL_MARGIN * col.normal.y;
            // Safety check, interpolation outside the unit range means no collision!
            // Helps to prevent glitches, it feels like?
            if col.interp_amount < 0.0 || col.interp_amount > 1.0 { return None };

            // use CollisionReaction::*;
            match reaction {
                CollisionReaction::Stop => {
                    // col.pos already contains the position the entity will be moved to
                },
                CollisionReaction::None => {
                    // TODO: No reaction means we should never refine, just getting the broad collision is enough?
                    // That will probable report false positives in fast moving objects, since the broad phase is too broad...
                    // Maybe additional parameters, like "Sweep" and "Instant" to account for fast objects,
                    
                    // Returns col.pos to its pre-collision state   
                    col.pos = self.collider.pos;
                },
                CollisionReaction::Bounce(amount) => {
                    // Redefines velocity
                    col.velocity = Vec2::reflect(result_velocity, col.normal).scale(amount);
                    // Experimental! Fixes reverse speed inheritance from collider in the axis orthogonal to collision?
                    col.velocity.x += 2.0 * other.velocity.x * col.normal.y.abs();
                    col.velocity.y += 2.0 * other.velocity.y * col.normal.x.abs();
                },
                CollisionReaction::Slide => {
                    let post_col_vel = {
                        let post_interp = 1.0 - col.interp_amount;
                        Vec2{
                            x: (self.velocity.x * post_interp) * col.normal.y.abs(),
                            y: (self.velocity.y * post_interp) * col.normal.x.abs()
                        }
                    };
                    col.pos.x += post_col_vel.x;
                    col.pos.y += post_col_vel.y;
                    col.velocity = post_col_vel;
                },
            }
            return Some(col);
        }

        None
    }


    // Performs collision checks using raycasts to obtain a collision normal and its location.
    // Assume broad AABB collision has already happened!
    fn refine_collision(&mut self, other_col:&Collider, tilemap:Option<&Tilemap>) -> Option<Collision<f32>> {
        match other_col.kind {
            ColliderKind::Point => {
                match self.collider.kind {
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
                let other_rect = Rect::from(*other_col);
                match self.collider.kind {
                    // Point to Rect
                    ColliderKind::Point => {
                        self.line_in_rect_collision(other_rect)
                    },
                    // Rect to Rect
                    ColliderKind::Rect{ .. } => {
                        Self::sweep_rect_to_rect_colllision(self.start_rect(), other_rect, self.velocity)
                    },
                    // Tilemap to Rect
                    ColliderKind::Tilemap { .. } => None,
                }
            },
            // Point to tilemap
            ColliderKind::Tilemap{ .. } => {
                let tilemap = tilemap?;
                let tilemap_rect = Rect::from(*other_col);

                let x0 = self.start_position.x - tilemap_rect.x;
                let y0 = self.start_position.y - tilemap_rect.y;
                let x1 = self.collider.pos.x - tilemap_rect.x;
                let y1 = self.collider.pos.y - tilemap_rect.y;

                match self.collider.kind {
                    // Point to Tilemap
                    ColliderKind::Point => {
                        tilemap.raycast(x0, y0, x1, y1)
                    },
                    // Rect to Tilemap
                    ColliderKind::Rect{ .. } => None,
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
            pos: Vec2{
                x: a.x + (vel_a.x * entry_time),
                y: a.y + (vel_a.y * entry_time),
            },
            normal,
            interp_amount: entry_time
        })
    }



    fn line_in_rect_collision(&self, rect:Rect<f32>) -> Option<Collision<f32>> {
        let trajectory = Ray { origin: self.collider.pos, angle: self.velocity.y.atan2(self.velocity.x) + PI };
        if let Some(mut col) = rect.intersect_ray(&trajectory){
            // TODO: This seems slower than necessary?
            // Maybe interset_ray can return the correct interpolation amount without further calculation?
            let len = self.velocity.len();
            if len > 0.0 {
                // println!("start:{:.2?}, end:{:.2?}", self.start_position, col.pos);
                let dist = self.start_position.distance_to(col.pos).abs();
                // println!("distance:{:.02?}", dist);
                col.interp_amount =  dist / len;
            }

            return Some(col)
        }
        None
    }


    // TODO: Return the other rect if true? It's goingto be re-used down the line
    fn broad_phase_overlaps(&self, other:&Self) -> bool {
        match self.collider.kind {
            ColliderKind::Point => {
                match other.collider.kind {
                    // Point in point
                    ColliderKind::Point => self.collider.pos.floor() == other.collider.pos.floor(),
                    // Point in rect
                    ColliderKind::Rect{ .. } | ColliderKind::Tilemap { .. }=> {
                        let rect = Rect::from(other.collider);
                        // Self::broad_phase_point_in_rect(self.start_position, self.collider.pos, rect)
                        Self::broad_phase_point_in_rect(self.start_position, self.velocity, rect, other.velocity)
                    },
                }
            },
            ColliderKind::Rect{ w, h } => {
                let rect = Rect {
                    x: self.start_position.x,
                    y: self.start_position.y,
                    w, h
                };
                match other.collider.kind {
                    // Rect over point
                    ColliderKind::Point => rect.contains(other.collider.pos.x, other.collider.pos.y),
                    // Rect over Rect
                    ColliderKind::Rect{ .. } | ColliderKind::Tilemap { .. }=> {
                        let other_rect = Rect::from(other.collider);
                        Self::broad_phase_rects_overlap(rect, other_rect, self.velocity)
                    },
                }
            },
            ColliderKind::Tilemap { .. } => {
                let rect = Rect::from(self.collider);
                match other.collider.kind {
                    // Rect over point
                    ColliderKind::Point => rect.contains(other.collider.pos.x, other.collider.pos.y),
                    // Rect over rect
                    ColliderKind::Rect{ .. } | ColliderKind::Tilemap { .. }=> {
                        let other_rect = Rect::from(other.collider);
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
        broad_rect.contains(point.x, point.y)

        // Simpler but imprecise method (does not sweep)
        // let rect_end = rect + rect_vel;
        // rect.contains(point.x, point.y) || rect.contains(point.x + point_vel.x, point.y + point_vel.y) ||
        // rect_end.contains(point.x, point.y) || rect_end.contains(point.x + point_vel.x, point.y + point_vel.y)
    }


    // fn broad_phase_point_in_rect(start: Vec2<f32>, end: Vec2<f32>, rect: Rect<f32>) -> bool {
    //     // rect sweeping method, needs more testing! May fail at high speeds, AABB sweep may be too broad?
    //     let delta = Vec2{
    //         x: start.x - end.x,
    //         y: start.y - end.y
    //     };
    //     let broad_rect = Self::broad_rect(rect, delta);
    //     broad_rect.contains(start.x, start.y)

    //     // Simpler but imprecise method (does not sweep)
    //     // rect.contains(start.x, start.y) || rect.contains(end.x, end.y)
    // }


    pub fn broad_phase_rects_overlap(a:Rect<f32>, b:Rect<f32>, vel_a:Vec2<f32>) -> bool {
        let broad_rect = Self::broad_rect(a, vel_a);
        broad_rect.overlaps(&b)
    }

}


impl From<Collider> for Rect<f32> {
    fn from(col: Collider) -> Self {
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