use num_traits::Float;
use crate::*;


pub struct Ray<T> {
    pub origin: Vec2<T>,
    pub angle: T, // In radians
}


#[derive(Debug, Default)]
pub struct Collision<T> where T:Float + PartialOrd + Copy{
    pub tile:Option<Tile>,
    pub point:Vec2<T>,
    pub normal:f32,
    pub collider_velocity:Vec2<T>   //TODO: output velocity after resolution, instead of collider velocity?
}


#[derive(Clone, Debug)]
pub struct Collider{
    // pub pos:Vec2<f32>,
    pub kind: ColliderKind,
    pub layer: u8,
    pub mask: u8
}


#[derive(Clone, Debug)]
#[repr(u8)]
pub enum ColliderKind{
    Point,
    Rect(Rect<f32>),
}


impl Collider {

    // Currently only 8 layers allowed
    fn get_layer_flags(layer_id:u8) -> u8 {
        if layer_id > 7 { panic!("Max 8 collision layers exceeded!") }
        2u8.pow(layer_id.into())
    }

    pub fn new_point(collision_layer:u8, collision_mask:u8) -> Self {
        Self {
            kind: ColliderKind::Point,
            layer: Self::get_layer_flags(collision_layer),
            mask: Self::get_layer_flags(collision_mask),
        }
    }


    pub fn new_rect(offset_x:f32, offset_y:f32, w:f32, h:f32, collision_layer:u8, collision_mask:u8) -> Self {
        Self {
            kind: ColliderKind::Rect(Rect { x: offset_x, y: offset_y, w, h }),
            layer: Self::get_layer_flags(collision_layer),
            mask: Self::get_layer_flags(collision_mask),
        }
    }

    // pub fn get_world_rect(&self, entity_x:f32, entity_y:f32) -> Rect {
    //     match self.kind {

    //     }
    // }
    
}

