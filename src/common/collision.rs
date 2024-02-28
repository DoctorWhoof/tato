use core::f32::consts::PI;
use num_traits::Float;
use crate::*;

const COL_MARGIN:f32 = 0.5;

#[derive(Debug)]
pub struct Ray<T> {
    pub origin: Vec2<T>,
    pub angle: T, // In radians
}


#[derive(Clone, Debug, Default)]
pub struct Collision<T> where T:Float + PartialOrd + Copy{
    pub tile:Option<Tile>,
    pub entity_id: EntityID,
    pub velocity:Vec2<T>,   //TODO: output velocity after resolution, instead of collider velocity?
    pub point:Vec2<T>,
    pub normal:f32,
}


#[derive(Clone, Debug)]
pub struct CollisionProbe<T> {
    pub entity_id: EntityID,
    pub collider:Collider,  // Contains the world space collider (obtained with entity.world_collider())
    pub start_position: Vec2<T>,
    pub velocity:Vec2<T>,
}


#[derive(Clone, Copy, Debug)]
pub struct Collider{
    pub enabled: bool,
    pub pos: Vec2<f32>,
    pub kind: ColliderKind,
    pub layer: u8,
    pub mask: u8
}


#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ColliderKind{
    Point,
    Rect{w:f32, h:f32},
    Tilemap{w:f32, h:f32, tile_width:u8, tile_height:u8}
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

    fn overlaps(&self, other:&Self) -> bool {
        match self.collider.kind {
            ColliderKind::Point => {
                match other.collider.kind {
                    ColliderKind::Point => self.collider.pos.floor() == other.collider.pos.floor(),
                    ColliderKind::Rect{ .. } | ColliderKind::Tilemap { .. }=> {
                        let rect = Rect::from(other.collider);
                        rect.contains(self.collider.pos.x, self.collider.pos.y)
                    },
                }
            },
            ColliderKind::Rect{ .. } => {
                let rect = Rect::from(self.collider);
                match other.collider.kind {
                    ColliderKind::Point => rect.contains(other.collider.pos.x, other.collider.pos.y),
                    ColliderKind::Rect{ .. } | ColliderKind::Tilemap { .. }=> {
                        let other_rect = Rect::from(other.collider);
                        rect.overlaps(&other_rect)
                    },
                }
            },
            ColliderKind::Tilemap { .. } => {
                let rect = Rect::from(self.collider);
                match other.collider.kind {
                    ColliderKind::Point => rect.contains(other.collider.pos.x, other.collider.pos.y),
                    ColliderKind::Rect{ .. } | ColliderKind::Tilemap { .. }=> {
                        let other_rect = Rect::from(other.collider);
                        rect.overlaps(&other_rect)
                    },
                }
            }
        }
    }

    fn line_in_rect_collision(&self, rect:Rect<f32>) -> Option<Collision<f32>> {
        let trajectory = Ray { origin: self.collider.pos, angle: self.velocity.y.atan2(self.velocity.x) + PI };
        if let Some((col_point, normal)) = rect.intersect_ray(&trajectory) {
            Some(Collision{
                tile: None,
                point: col_point,
                normal,
                entity_id: self.entity_id,
                velocity: self.velocity,
            })
        } else {
            None
        }
    }

    // Performs collision checks using raycasts to obtain a collision normal and its location.
    // Assume broad AABB collision has already happened!
    fn refine_collision(&mut self, other_col:&Collider, tilemap:Option<&Tilemap>) -> Option<Collision<f32>> {
        match self.collider.kind {
            ColliderKind::Point => {
                match other_col.kind {
                    // Point to point
                    ColliderKind::Point => {
                        todo!()
                    },
                    // Point to Rect
                    ColliderKind::Rect{..} => {
                        let other_rect = Rect::from(*other_col);
                        self.line_in_rect_collision(other_rect)
                    },
                    // Point to tilemap
                    ColliderKind::Tilemap{ tile_width, tile_height, .. } => {
                        let tilemap = tilemap?;
                        let tile_width = tile_width as f32;
                        let tile_height = tile_height as f32;
                        let tilemap_rect = Rect::from(*other_col);
                        // let x0 = ((self.start_position.x - tilemap_rect.x) / tile_width) as i32;
                        // let y0 = ((self.start_position.y - tilemap_rect.y) / tile_height) as i32;
                        // let x1 = ((self.collider.pos.x - tilemap_rect.x) / tile_width) as i32;
                        // let y1 = ((self.collider.pos.y - tilemap_rect.y) / tile_height) as i32;
                        let x0 = (self.start_position.x - tilemap_rect.x) / tile_width;
                        let y0 = (self.start_position.y - tilemap_rect.y) / tile_height;
                        let x1 = (self.collider.pos.x - tilemap_rect.x) / tile_width;
                        let y1 = (self.collider.pos.y - tilemap_rect.y) / tile_height;
                        // TODO: Use intersection point!!!!
                        // tilemap.raycast(x0, y0, x1, y1)
                        if let Some(mut col) = tilemap.raycast(x0, y0, x1, y1) {
                            // col.velocity = Vec2::reflect(self.velocity, col.normal);
                            col.point.x *= tile_width;
                            col.point.y *= tile_height;
                            return Some(col)
                        }
                        // if let Some(col) = tilemap.raycast(x0, y0, x1, y1){
                        //     let col_rect = Rect{
                        //         x: col.point.x * tile_width,
                        //         y: col.point.y * tile_height,
                        //         w: tile_width,
                        //         h: tile_height
                        //     };
                        //     println!("rect after processing: {:?}", col_rect);
                        //     return self.line_in_rect_collision(col_rect)
                        // }
                        None
                    },
                }
            }
            ColliderKind::Rect{..} | ColliderKind::Tilemap { .. } => { // no "Tilemap to other" collisions yet, only the opposite
                None
            }
        }
    }


    pub fn collision_response(&self, other:&Self, bounce:f32, tilemap:Option<&Tilemap>, _elapsed:f32) -> Option<Collision<f32>> {
        if !self.overlaps(other) { return None }

        // Turns the incoming collider velocity into additional self velocity
        let result_velocity = Vec2::weighted_add(self.velocity, other.velocity, 1.0, -1.0);


        let mut probe = self.clone();
        probe.velocity = result_velocity;
        if let Some(mut col) = probe.refine_collision(&other.collider, tilemap) {
            // println!("collision after processing: {:?}", col);

            let normal_x = col.normal.cos();
            let normal_y = col.normal.sin();

            let x = col.point.x + COL_MARGIN * normal_x;
            let y = col.point.y - COL_MARGIN * normal_y;

            col.point.x = lerp(self.collider.pos.x, x, normal_x.abs());
            col.point.y = lerp(self.collider.pos.y, y, normal_y.abs());

            col.velocity = Vec2::reflect(result_velocity, col.normal).scale(bounce);

            // let mut secondary_probe = self.clone();
            // secondary_probe.velocity = result_velocity
            // let mut secondary_probe = CollisionProbe{
            //     entity_id: self.entity_id,
            //     collider: self.collider,
            //     start_position: col.point,
            //     velocity: col.velocity,
            // };
            // secondary_probe.collider.pos = col.point + (col.velocity.scale(_elapsed));
            // if let Some(_new_col) = secondary_probe.refine_collision(&other.collider, tilemap) {
            //     println!("Secondary collision");
            //     // col.point = self.start_position;
            //     // return Some(Collision{
            //     //     tile: None,
            //     //     entity_id: self.entity_id,
            //     //     velocity: Vec2::reflect(col.velocity, col.normal).scale(bounce),
            //     //     point: self.start_position,
            //     //     normal: new_col.normal,
            //     // })
            // }
            return Some(col);
        }

        // // Resolution in X
        // let mut probe_x = self.clone();
        // probe_x.velocity = result_velocity;
        // probe_x.collider.pos.y -= result_velocity.y * elapsed;
        // let result_x = probe_x.refine_collision(&other.collider, tilemap);

        // if let Some(mut col) = result_x {
        //     let margin = COL_MARGIN * result_velocity.x.signum();
        //     col.point.x -= margin;
        //     col.point.y = self.collider.pos.y;
        //     col.velocity = Vec2::reflect(col.velocity, col.normal).scale(bounce);
        //     return Some(col);
        // }

        // // Resolution in Y
        // let mut probe_y = self.clone();
        // probe_y.velocity = result_velocity;
        // probe_y.collider.pos.x -= result_velocity.x * elapsed;
        // let result_y = probe_y.refine_collision(&other.collider, tilemap); 

        // if let Some(mut col) = result_y {
        //     let margin = COL_MARGIN * result_velocity.y.signum();
        //     col.point.x = self.collider.pos.x;
        //     col.point.y -= margin;
        //     col.velocity = Vec2::reflect(col.velocity, col.normal).scale(bounce);
        //     return Some(col);
        // }

        None
    }
}