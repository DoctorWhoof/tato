use crate::*;
use slotmap::new_key_type;


new_key_type! {
    /// A key to the World slotmap containing entities.
    pub struct EntityID;
}

/// Contains the necessary structs to provide rendering and collisions
#[derive(Clone, Debug, Default)]
pub struct Entity {
    // Accessible to "engine" only, not to the host game. Can't be changed after creation.
    pub(super) id: EntityID,

    // Public
    pub depth: u8,      // 0 means "background", higher values render on top
    pub visible: bool, //TODO: Flags? Would allow mirroring whole entity, with colliders and offset properly mirrored.
    pub shape: Shape,
    pub pos: Vec2<f32>,
    pub render_offset: Vec2<i8>,
}

impl Entity {
    pub fn id(&self) -> EntityID {
        self.id
    }

    pub fn new(with_id: EntityID, depth:u8) -> Entity {
        Self {
            depth,
            id: with_id,
            visible: true,
            shape: Shape::None,
            pos: Vec2 { x: 0.0, y: 0.0 },
            render_offset: Vec2 { x: 0, y: 0 },
        }
    }

    // fn world_vec2<T:Into<f32>>(&self, pos: Vec2<T>, use_render_offset: bool) -> Vec2<f32> {
    //     Vec2 {
    //         x: if use_render_offset {
    //             self.pos.x + pos.x.into() + self.render_offset.x as f32
    //         } else {
    //             self.pos.x + pos.x.into()
    //         },
    //         y: if use_render_offset {
    //             self.pos.y + pos.y.into() + self.render_offset.y as f32
    //         } else {
    //             self.pos.y + pos.y.into()
    //         },
    //     }
    // }

    // pub fn world_rect<T: Into<f32>>(&self, rect: Rect<T>, use_render_offset: bool) -> Rect<f32> {
    //     let pos = self.world_vec2(
    //         Vec2 {
    //             x: rect.x,
    //             y: rect.y,
    //         },
    //         use_render_offset,
    //     );
    //     Rect {
    //         x: pos.x,
    //         y: pos.y,
    //         w: rect.w.into(),
    //         h: rect.h.into(),
    //     }
    // }

}
