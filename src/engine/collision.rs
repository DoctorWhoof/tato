use core::ops::{Add, AddAssign};
use crate::{CollisionLayer, EntityID, Rect, Tile, Tilemap, Vec2, Float, optional_sum};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Axis {
    Horizontal,
    Vertical,
    Both
}


#[derive(Debug, Clone, Copy)]
pub enum CollisionReaction {
    None,
    Bounce(f32),
    Slide
}


#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ColliderKind<T>
where T: Float {
    Point,
    Rect{ w:T, h:T },
    Tilemap{ w:T, h:T, tile_width:u8, tile_height:u8 } // Is populated by World, values depend on the tilemap
}


/// Optional data included in a collision result
#[derive(Clone, Copy, Debug)]
pub struct TileCollision {
    pub tile:Tile,
    pub row:u16,
    pub col:u16
}

/// The partial result of a single axis collision
#[derive(Clone, Copy, Debug, Default)]
struct PartialCollision<T>
where T:Float + PartialOrd + Copy {
    pos: T,
    normal: T,
}

/// The result of a collision
#[derive(Clone, Copy, Debug)]
pub struct Collision<T>
where T:Float + PartialOrd + Copy {
    pub t: Vec2<T>,
    pub pos: Vec2<T>,
    pub normal: Vec2<T>,
    pub velocity:Vec2<T>,
    pub colliding_entity: EntityID,
    pub tile: Option<TileCollision>,
}


/// Allows an entity to specify a collision shape, a position offset and collision layer masking.
#[derive(Clone, Copy, Debug)]
pub struct Collider<T>
where T: Float {
    pub kind: ColliderKind<T>,
    pub offset: Vec2<T>,
    pub enabled: bool,
    pub mask: u16,
    pub(crate) layer: u16,  // Set by World, semi-private so that it can't be changed
}

/// Generated when checking for collisions, contains additional collider information like velocity and start position.
#[derive(Clone, Debug)]
pub struct CollisionProbe<T>
where T: Float {
    pub kind: ColliderKind<T>,
    pub pos: Vec2<T>,
    pub entity_id: EntityID,
    pub velocity:Vec2<T>,
    pub layer: u16,
    pub mask: u16
}



impl<T> Collider<T>
where T: Float {

    pub fn from_tilemap(layer:impl CollisionLayer) -> Self {
        Self {
            enabled: true,
            offset: Vec2::zero(),
            kind: ColliderKind::Tilemap { w:T::zero(), h:T::zero(), tile_width:0, tile_height:0 }, // Values will be written by World
            layer: layer.into(),
            mask: 0,
        }
    }

    pub fn from_point(layer:impl CollisionLayer, x:T, y:T) -> Self {
        Self {
            enabled: true,
            offset: Vec2 { x, y },
            kind: ColliderKind::Point,
            layer: layer.into(),
            mask: 0,
        }
    }

    pub fn from_rect(layer:impl CollisionLayer, rect:Rect<T>) -> Self {
        Self {
            enabled: true,
            offset: Vec2 { x:rect.x, y:rect.y },
            kind: ColliderKind::Rect{ w:rect.w, h:rect.h },
            layer: layer.into(),
            mask: 0,
        }
    }

}



impl<T> CollisionProbe<T>
where T: Float {

    fn sweep_rect(a:Rect<T>, vel:Vec2<T>) -> Rect<T> {
        Rect{
            x: if vel.x > T::zero() { a.x } else { a.x + vel.x },
            y: if vel.y > T::zero() { a.y } else { a.y + vel.y },
            w: if vel.x > T::zero() { vel.x + a.w } else { a.w - vel.x },
            h: if vel.y > T::zero() { vel.y + a.h } else { a.h - vel.y },
        }
    }


    pub(crate) fn collision_response(&self, other:&CollisionProbe<T>, tilemap:Option<&Tilemap>) -> Option<Collision<T>> {

        let vel_result = other.velocity - self.velocity;

        // Self will ALWAYS be reduced to a point, and other will ALWAYS be expanded to include self size
        let size = match self.kind {
            ColliderKind::Point => Vec2::zero(),
            ColliderKind::Rect { w, h } | ColliderKind::Tilemap { w, h, .. } => Vec2 { x: w, y: h },
        };

        let mut expanded_rect = Rect::from(other);
        expanded_rect.x -= size.x;
        expanded_rect.y -= size.y;
        expanded_rect.w += size.x;
        expanded_rect.h += size.y;

        let broad_rect = Self::sweep_rect(expanded_rect, vel_result);

        if broad_rect.contains(self.pos.x, self.pos.y) {
            match other.kind {
                ColliderKind::Point => { return None },

                ColliderKind::Rect { .. } => {
                    // X Step
                    let rect_x = Self::sweep_rect(expanded_rect, vel_result.horiz());
                    let col_x = if rect_x.contains(self.pos.x, self.pos.y) {
                        if vel_result.x > T::zero() {
                            Some(PartialCollision{ pos:expanded_rect.right(), normal: T::one() })
                        } else if vel_result.x < T::zero() {
                            Some(PartialCollision{ pos:expanded_rect.x, normal: -T::one() })
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Y Step
                    let rect_y = Self::sweep_rect(expanded_rect, vel_result.vert());
                    let col_y = if rect_y.contains(self.pos.x, self.pos.y) {
                        if vel_result.y > T::zero() {
                            Some(PartialCollision{ pos:expanded_rect.bottom(), normal: T::one() })
                        } else if vel_result.y < T::zero() {
                            Some(PartialCollision{ pos:expanded_rect.y, normal: -T::one() })
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Combine results from both axes (instead of returning early on single axis collision).
                    // Allows for correct corner collisions, but seems less stable on moving platforms.
                    if col_x.is_some() || col_y.is_some() {
                        let col_x = col_x.unwrap_or({
                            // No X collision
                            PartialCollision { pos: self.pos.x + self.velocity.x, normal:T::zero() }
                        });
                        let col_y = col_y.unwrap_or({
                            // No Y collision
                            PartialCollision { pos: self.pos.y + self.velocity.y, normal:T::zero() }
                        });
                        return Some(Collision{
                            t: Vec2 {
                                x:if self.velocity.x != T::zero() {
                                    (col_x.pos - self.pos.x) / self.velocity.x
                                } else {
                                    self.velocity.x
                                },
                                y:if self.velocity.y != T::zero() {
                                    (col_y.pos - self.pos.y) / self.velocity.y
                                } else {
                                    self.velocity.y
                                },
                            },
                            pos: Vec2 { x:col_x.pos, y:col_y.pos },
                            normal: Vec2 { x:col_x.normal, y:col_y.normal },
                            velocity: other.velocity,
                            colliding_entity: other.entity_id,
                            tile: None,
                        })
                    } else {
                        return None
                    }
                },

                ColliderKind::Tilemap { tile_width, tile_height, .. } => {
                    let tilemap = tilemap?;
                    let tile_width = T::from_u8(tile_width)?;
                    let tile_height = T::from_u8(tile_height)?;
                    let tilemap_rect = Rect::from(other);
                    
                    // "vel_result" has self in reverse, must be negated.
                    // We could just use self.velocity, since the tilemap will never be moving... or will it?
                    let vel_x = -vel_result.x / tile_width;
                    let vel_y = -vel_result.y / tile_height;
    
                    // Top left coords
                    let x0 = (self.pos.x - tilemap_rect.x) / tile_width;
                    let y0 = (self.pos.y - tilemap_rect.y) / tile_height;

                    use Axis::*;
                    match self.kind {
                        // Point to Tilemap
                        ColliderKind::Point => {
                            return tilemap.collide_adjacent(x0, y0, x0 + vel_x, y0 + vel_y, Both)
                        },
                        // Rect to tilemap
                        ColliderKind::Rect { w, h } | ColliderKind::Tilemap { w, h, .. } => {
                            // Warning: Rect collision samples points around the rect. 
                            // Will fail if the collider is larger than tile dimensions * 2.0!
                            let w = w / tile_width;
                            let h = h / tile_width;
                            
                            // Get appropriate a,b,c corners depending on velocity
                            let (a, axis_a, b, axis_b, c, axis_c) = if vel_x.is_sign_positive() {
                                if vel_y.is_sign_positive(){(
                                    Vec2{ x:x0 + w  , y:y0      }, Horizontal,      // TR
                                    Vec2{ x:x0 + w  , y:y0 + h  }, Both,            // BR
                                    Vec2{ x:x0      , y:y0 + h  }, Vertical,        // BL
                                )} else {(
                                    Vec2{ x:x0      , y:y0      }, Vertical,        // TL
                                    Vec2{ x:x0 + w  , y:y0      }, Both,            // TR
                                    Vec2{ x:x0 + w  , y:y0 + h  }, Horizontal       // BR
                                )}
                            } else if vel_y.is_sign_positive(){(
                                Vec2{ x:x0      , y:y0      }, Horizontal,          // TL
                                Vec2{ x:x0      , y:y0 + h  }, Both,                // BL
                                Vec2{ x:x0 + w  , y:y0 + h  }, Vertical,            // BR
                            )} else {(
                                Vec2{ x:x0 + w  , y:y0      }, Vertical,            // TR
                                Vec2{ x:x0      , y:y0      }, Both,                // TL
                                Vec2{ x:x0      , y:y0 + h  }, Horizontal,          // BL
                            )};

                            let col_a = tilemap.collide_adjacent(a.x, a.y, a.x + vel_x, a.y + vel_y, axis_a);
                            let col_b = tilemap.collide_adjacent(b.x, b.y, b.x + vel_x, b.y + vel_y, axis_b);
                            let col_c = tilemap.collide_adjacent(c.x, c.y, c.x + vel_x, c.y + vel_y, axis_c);

                            // Height is more than a tile
                            if h > T::one() {
                                let step_delta = h / h.ceil();
                                let mut step = step_delta;
                                while step < h {
                                    let x = if vel_x.is_sign_positive(){ x0 + w } else { x0 };
                                    let y = y0 + step;
                                    let vert_col = tilemap.collide_adjacent(x, y, x + vel_x, y, Horizontal);
                                    if vert_col.is_some(){
                                        return optional_sum(&[col_a, col_b, col_c, vert_col]);
                                    }
                                    step += step_delta;
                                } 
                            };

                            // Width is more than a tile
                            if w > T::one() {
                                let step_delta = w / w.ceil();
                                let mut step = step_delta;
                                while step < w {
                                    let x = x0 + step;
                                    let y = if vel_y.is_sign_positive(){ y0 + h } else { y0 };
                                    let horz_col = tilemap.collide_adjacent(x, y, x, y + vel_y, Vertical);
                                    if horz_col.is_some(){
                                        return optional_sum(&[col_a, col_b, col_c, horz_col]);
                                    }
                                    step += step_delta;
                                } 
                            };

                            // No edge collisions, return average result from (a,b,c) corners
                            return optional_sum(&[col_a, col_b, col_c]);
                        },
                    }
                },
            }
        }

        None
    }

}


impl<T> From<&CollisionProbe<T>> for Rect<T>
where T: Float {
    fn from(col: &CollisionProbe<T>) -> Rect<T> {
        let (w,h) = match col.kind {
            ColliderKind::Point => (T::zero(), T::zero()),
            ColliderKind::Rect { w, h } => ( w, h ),
            ColliderKind::Tilemap { w, h, .. } => ( w, h )
        };
        Rect{x: col.pos.x, y: col.pos.y, w, h}
    }
}


impl<T> Add for Collision<T>
where T:Float {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            t: Vec2 {
                x: self.t.x.min(other.t.x),
                y: self.t.y.min(other.t.y)
            },
            pos: self.pos.average(&other.pos),
            normal: self.normal + other.normal,
            velocity: self.velocity + other.velocity,
            colliding_entity: self.colliding_entity,
            tile: self.tile,
        }
    }
}

impl<T> AddAssign for Collision<T>
where T:Float {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
