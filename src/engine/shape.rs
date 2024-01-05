use crate::*;

/// Determines how an entity's rectangle is rendered to pixels.
#[derive(Clone, Debug)]
pub enum Shape {
    None,
    // Invisible,
    Sprite {     // Sprites can be placed anywhere in the world
        anim_id:u8,
        flip_h:bool,
        flip_v:bool
    },   
    AnimTiles {  // Renders its tile data to a tilemap, instead of the world
        anim_id:u8,
        tilemap_entity:EntityID,
        flip_h:bool,
        flip_v:bool,
    }, 
    TilemapLayer {  // Provides a world rectagle in which to draw a tilemap
        tilemap_id:u8
    },
}


impl Shape {

    pub fn sprite_from_anim(anim_id:u8) -> Self {
        Shape::Sprite {
            anim_id,
            flip_h: false,
            flip_v: false,
        }
    }


    pub fn anim_tiles_from_anim(anim_id:u8, tilemap_entity:EntityID) -> Self {
        Shape::AnimTiles {
            anim_id,
            flip_h: false,
            flip_v: false,
            tilemap_entity,
        }
    }
    
}