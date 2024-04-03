use crate::{CollisionLayer, EntityID, Rect, Tile, Tilemap, Vec2, Float};

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Axis {
    Horizontal,
    Vertical
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
#[derive(Clone, Copy, Debug, Default)]
pub struct Collision<T>
where T:Float + PartialOrd + Copy {
    // pub axis: Axis,
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
    pub pos: Vec2<T>,
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

    pub fn new_tilemap_collider(layer:impl CollisionLayer) -> Self {
        Self {
            enabled: true,
            pos: Vec2::zero(),
            kind: ColliderKind::Tilemap { w:T::zero(), h:T::zero(), tile_width:0, tile_height:0 }, // Values will be written by World
            layer: layer.into(),
            mask: 0,
        }
    }

    pub fn new_point_collider(layer:impl CollisionLayer, x:T, y:T) -> Self {
        Self {
            enabled: true,
            pos: Vec2 { x, y },
            kind: ColliderKind::Point,
            layer: layer.into(),
            mask: 0,
        }
    }

    pub fn new_rect_collider(layer:impl CollisionLayer, rect:Rect<T>) -> Self {
        Self {
            enabled: true,
            pos: Vec2 { x:rect.x, y:rect.y },
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
                    // In practice we could just use self.velocity, since the tilemap will never be moving... or will it?
                    let vel_x = -vel_result.x / tile_width;
                    let vel_y = -vel_result.y / tile_height;
    
                    // Point to Tilemap
                    let x0 = (self.pos.x - tilemap_rect.x) / tile_width;
                    let y0 = (self.pos.y - tilemap_rect.y) / tile_height;
                    let x1 = x0 + vel_x;
                    let y1 = y0 + vel_y;
                    
                    // // For now at least, rects will be sampled at multiple points...
                    // let (cols, rows) = match self.kind {
                    //     ColliderKind::Point | ColliderKind::Tilemap{ .. } => (1,1),
                    //     ColliderKind::Rect { w, h } => (
                    //         ((w / tile_width).ceil() + T::one()).to_u8()?,
                    //         ((h / tile_height).ceil() + T::one()).to_u8()?
                    //     ),
                    // };

                    // TODO: Simplified horiz_cast and vert_cast, limited to a single neightbor tile!
                    // TODO: Rect to tilemap (only raycast outer points!). Use simpler cast, i.e. middle points can use single axis

                    let col_x = tilemap.raycast(x0, y0, x1, y0);
                    let col_y = tilemap.raycast(x0, y0, x0, y1);

                    if col_x.is_some() && col_y.is_some() {
                        let col_x = col_x.unwrap();
                        let mut col_y = col_y.unwrap();
                        col_y.normal.x = col_x.normal.x;
                        col_y.t.x = col_x.t.x;
                        return Some(col_y)
                    }
                    if col_x.is_some() { return col_x }
                    if col_y.is_some() { return col_y }

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


// // TEST: Early Y return
// if let Some(col) = col_y {
//     return Some(Collision{
//         t: Vec2 {
//             x: T::one(),
//             y:if self.velocity.y != T::zero() {
//                 (col.pos - self.pos.y) / self.velocity.y
//             } else {
//                 self.velocity.y
//             },
//         },
//         pos: Vec2 { x:self.pos.x + self.velocity.x, y:col.pos },
//         normal: Vec2 { x:T::zero(), y:col.normal },
//         velocity: other.velocity,
//         colliding_entity: other.entity_id,
//         tile: None,
//     })
// }

// // TEST: Early X return
// // Assumes AABB collisions are always single axis, but can fail at corners...
// if let Some(col) = col_x {
//     return Some(Collision{
//         t: Vec2 {
//             x:if self.velocity.x != T::zero() {
//                 (col.pos - self.pos.x) / self.velocity.x
//             } else {
//                 self.velocity.x
//             },
//             y: T::one(),
//         },
//         pos: Vec2 { x:col.pos, y:self.pos.y + self.velocity.y },
//         normal: Vec2 { x:col.normal, y:T::zero() },
//         velocity: other.velocity,
//         colliding_entity: other.entity_id,
//         tile: None,
//     })
// }