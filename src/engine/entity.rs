use crate::*;

#[derive(Clone, Debug)]
pub struct Entity {
    // Accessible to "engine" only, not to "Game", so that only World can set it.
    pub(super) id:EntityID,     

    // Public
    pub shape: Shape,
    pub pos: Vec2<f32>,
    pub col: Option<Rect<i8>>,
    pub render_offset: Vec2<i8>,
}


impl Default for Entity {
    fn default() -> Self {
        Self { 
            id: EntityID(0),    // Points to 
            shape: Shape::None, 
            pos: Vec2 { x: 0.0, y: 0.0 }, 
            // vel: Vec2 { x: 0.0, y: 0.0 }, 
            render_offset: Vec2 { x: 0, y: 0 },
            col: None, 
        }
    }
}

impl Entity {

    pub fn id(&self) -> EntityID { self.id }

    fn world_vec2<T>(&self, pos:Vec2<T>, use_render_offset:bool) -> Vec2<f32>
    where T:Into<f32> {
        Vec2{
            x: if use_render_offset {
                self.pos.x + pos.x.into() + self.render_offset.x as f32 
            } else {
                self.pos.x + pos.x.into()
            },
            y: if use_render_offset {
                self.pos.y + pos.y.into() + self.render_offset.y as f32 
            } else {
                self.pos.y + pos.y.into()
            }
        }
    }


    pub fn world_rect<T:Into<f32>>(&self, rect:Rect<T>, use_render_offset:bool) -> Rect<f32> {
        let pos = self.world_vec2(Vec2{x:rect.x, y:rect.y}, use_render_offset);
        Rect{ x:pos.x, y:pos.y, w: rect.w.into(), h: rect.h.into() }
    }


}