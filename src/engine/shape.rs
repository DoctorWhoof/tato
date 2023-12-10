use crate::*;


#[derive(Clone, Debug)]
pub enum Shape {
    None,
    // Invisible,
    Sprite {     // Sprites can be placed anywhere in the world
        anim_id:AnimID,
        flip_h:bool,
        flip_v:bool
    },   
    AnimTiles {  // Renders its tile data to a tilemap, instead of the world
        anim_id:AnimID,
        tilemap_entity:EntityID,
        flip_h:bool,
        flip_v:bool,
    }, 
    TilemapLayer {  // Provides a world rectagle in which to draw a tilemap
        tilemap_id:TilemapID
    },
}


impl Shape {

    pub fn sprite_from_anim(anim_id:AnimID) -> Self {
        Shape::Sprite {
            anim_id,
            flip_h: false,
            flip_v: false,
        }
    }


    pub fn anim_tiles_from_anim(anim_id:AnimID, tilemap_entity:EntityID) -> Self {
        Shape::AnimTiles {
            anim_id,
            flip_h: false,
            flip_v: false,
            tilemap_entity,
        }
    }
    
}