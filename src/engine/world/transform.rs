use crate::*;


impl<T,P> World<T,P>
where T:TilesetEnum, P:PaletteEnum {

    
    pub fn get_position(&self, id: EntityID) -> Vec2<f32> {
        self.entities[id].pos
    }


    pub fn set_position(&mut self, id: EntityID, pos:Vec2<f32>) {
        self.entities[id].pos = pos;
    }

    /// Takes the frame delta time into account, but does not perform collisions.
    pub fn translate(&mut self, id:EntityID, delta:Vec2<f32>) {
        self.entities[id].pos += delta.scale(self.time_elapsed);
    }


    pub fn get_entity_rect_from_id(&self, id: EntityID) -> Rect<f32> {
        if let Some(entity) = self.get_entity(id) {
            self.get_entity_rect(entity)
        } else {
            Rect::default()
        }
    }


    pub fn get_entity_rect(&self, entity: &Entity) -> Rect<f32> {
        match entity.shape {
            Shape::None => Default::default(),
            Shape::Sprite {tileset_id, anim_id, ..} | Shape::BgSprite {tileset_id, anim_id, ..} => {
                let anim = self.renderer.get_anim(tileset_id, anim_id);
                let frame = anim.frame(self.time);
                Rect {
                    x: entity.pos.x + entity.render_offset.x as f32,
                    y: entity.pos.y + entity.render_offset.y as f32,
                    w: (frame.cols as usize * self.specs.tile_width as usize) as f32,
                    h: (frame.rows as usize * self.specs.tile_height as usize) as f32,
                }
            }
            Shape::Bg {tileset_id,tilemap_id} => {
                let tilemap = &self.renderer.get_tilemap(tileset_id, tilemap_id);
                Rect {
                    x: entity.pos.x + entity.render_offset.x as f32,
                    y: entity.pos.y + entity.render_offset.y as f32,
                    w: tilemap.width(self.specs.tile_width) as f32,
                    h: tilemap.height(self.specs.tile_height) as f32,
                }
            }
        }
    }


    pub fn get_tilemap_and_rect(&self, id: EntityID) -> Option<(&Tilemap, Rect<f32>)> {
        let tilemap_entity = self.get_entity(id)?;
        let Shape::Bg {
            tileset_id,
            tilemap_id,
        } = tilemap_entity.shape
        else {
            return None;
        };
        let tilemap = &self.renderer.get_tilemap(tileset_id, tilemap_id);
        Some((tilemap, self.get_entity_rect(tilemap_entity)))
    }



}