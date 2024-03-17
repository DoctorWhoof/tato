use crate::*;

/// Determines how an entity's rectangle is rendered to pixels.
#[derive(Clone, Debug, Default)]
pub enum Shape {
    #[default]
    None,
    // Invisible,
    Sprite {     // Sprites can be placed anywhere in the world
        tileset_id:u8,
        anim_id:u8,
        flip_h:bool,
        flip_v:bool
    },
    BgSprite {  // Renders its tile data to a tilemap, instead of the world
        tileset_id:u8,
        anim_id:u8,
        tilemap_entity:EntityID,
        flip_h:bool,
        flip_v:bool,
    },
    Bg {  // Allows an entity to draw a tilemap
        tileset_id:u8,
        tilemap_id:u8
    },
}


impl Shape {

    pub fn sprite_from_anim(tileset_id:impl TilesetEnum, anim_id:impl Into<u8>) -> Self {
        Shape::Sprite {
            tileset_id: tileset_id.into(),
            anim_id: anim_id.into(),
            flip_h: false,
            flip_v: false,
        }
    }


    pub fn anim_tiles_from_anim(tileset_id:impl TilesetEnum, anim_id:impl Into<u8>, tilemap_entity:EntityID) -> Self {
        Shape::BgSprite {
            tileset_id: tileset_id.into(),
            anim_id: anim_id.into(),
            flip_h: false,
            flip_v: false,
            tilemap_entity,
        }
    }

}
