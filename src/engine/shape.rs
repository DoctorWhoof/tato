use crate::*;

/// Determines how an entity's rectangle is rendered to pixels.
#[derive(Clone, Debug, Default)]
pub enum Shape {
    #[default]
    None,
    // Invisible,
    Sprite {     // Sprites can be placed anywhere in the world
        anim_id:u8,
        flip_h:bool,
        flip_v:bool
    },   
    BgTiles {  // Renders its tile data to a tilemap, instead of the world
        anim_id:u8,
        tilemap_entity:EntityID,
        flip_h:bool,
        flip_v:bool,
    }, 
    Bg {  // Allows an entity to draw a tilemap
        tilemap_id:u8
    },
}


impl Shape {

    pub fn sprite_from_anim(anim_id:impl Into<u8>) -> Self {
        let anim_id = match anim_id.try_into(){
            Ok(value) => value,
            Err(_) => panic!("Invalid anim_id"),
        };
        Shape::Sprite {
            anim_id,
            flip_h: false,
            flip_v: false,
        }
    }


    pub fn anim_tiles_from_anim(anim_id:impl Into<u8>, tilemap_entity:EntityID) -> Self {
        let anim_id = match anim_id.try_into(){
            Ok(value) => value,
            Err(_) => panic!("Invalid anim_id"),
        };
        Shape::BgTiles {
            anim_id,
            flip_h: false,
            flip_v: false,
            tilemap_entity,
        }
    }
    
}