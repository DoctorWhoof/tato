use crate::*;


impl<T,P> World<T,P>
where T:TilesetEnum, P:PaletteEnum {

    // **************************************** Render ***************************************


    pub fn is_visible(&mut self, id:EntityID) -> bool {
        self.entities[id].visible
    }


    pub fn center_camera_on(&mut self, id: EntityID) {
        let Some(ent) = self.get_entity(id) else {
            return;
        };
        let pos = ent.pos;
        self.cam.x = pos.x - (self.framebuf.width() / 2) as f32;
        self.cam.y = pos.y - (self.framebuf.height() / 2) as f32;
    }


    pub fn set_visible(&mut self, id:EntityID, visible:bool) {
        self.entities[id].visible = visible;
    }


    pub fn set_viewport(&mut self, rect: Rect<i32>) {
        self.framebuf.viewport = rect;
        self.cam.w = rect.w as f32;
        self.cam.h = rect.h as f32;
    }


    pub fn set_render_offset(&mut self, id: EntityID, x:i8, y:i8) {
        self.entities[id].render_offset = Vec2 { x, y };
    }


    pub fn set_shape(&mut self, id:EntityID, shape:Shape) {
        self.entities[id].shape = shape;
    }


    // ************************************* Drawing functions ***********************************


    pub(crate) fn draw_collider(framebuf:&mut FrameBuf, cam_rect:&Rect<i32>, probe:&CollisionProbe<f32>, color:Color24){
        match probe.kind {
            ColliderKind::Point =>{
                let pos = probe.pos.to_i32();
                if cam_rect.contains(pos.x, pos.y) {
                    framebuf.draw_pixel(pos.x as usize, pos.y as usize, color, 255);
                }
            },
            ColliderKind::Rect{..} | ColliderKind::Tilemap{..} =>{
                let rect = Rect::from(probe).to_i32();
                let screen_col = rect - cam_rect.pos();
                if cam_rect.overlaps(&rect) {
                    framebuf.draw_rect(screen_col, color);
                }
            },
        }
    }


    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, info: &FontInfo) {
        let font = &self.renderer.get_font(info.tileset_id, info.font as usize);
        for (i, letter) in text.chars().enumerate() {
            let letter = letter as u32;
            let index = if letter > 47 {
                if letter < 65 {
                    (letter - 48) as u16 + font.start_index as u16 // Zero
                } else {
                    (letter - 55) as u16 + font.start_index as u16 // Upper Case 'A' (A index is 65, but the first 10 tiles are the numbers so we add 10)
                }
            } else {
                font.last() as u16 // Space
            };

            let offset_x = if info.align_right {
                (self.specs.tile_width as usize * text.len()) as i32
            } else {
                0
            };

            let abs_tile_id = self
                .renderer
                .get_tile(u8::try_from(index).unwrap(), font.tileset_id as usize);

            Self::draw_tile(
                &mut self.framebuf,
                &self.renderer,
                Rect {
                    x: x + (i * self.specs.tile_width as usize) as i32 - offset_x,
                    y,
                    w: self.specs.tile_width as i32,
                    h: self.specs.tile_height as i32,
                },
                abs_tile_id,
                self.renderer.get_tileset_palette(info.tileset_id),
                false,
                info.depth
            )
        }
    }


    pub(crate) fn draw_tile(
        frame_buf: &mut FrameBuf,
        renderer: &Renderer<T,P>,
        world_rect: Rect<i32>,
        tile: TileID,
        palette: &Palette,
        flip_h: bool,
        depth:u8
    ) {
        let Some(visible_rect) = world_rect.intersect(frame_buf.viewport) else {
            // println!("Out of frame");
            return;
        };
        let tile_rect = renderer.get_rect(tile.get());
        let width = frame_buf.width();

        for y in visible_rect.y .. visible_rect.bottom() {
            let source_y = y - world_rect.y;
            #[cfg(debug_assertions)]{
                if source_y < 0 { panic!("Whoops, coordinate can't be negative!") }
            }
            let source_y = source_y as usize + tile_rect.y as usize;

            for x in visible_rect.x .. visible_rect.right() {
                let source_x = if flip_h {
                    let local_x = renderer.tile_width() as usize - (x - world_rect.x) as usize - 1;
                    local_x + tile_rect.x as usize
                } else {
                    let local_x = (x - world_rect.x) as usize;
                    local_x + tile_rect.x as usize
                };

                let color = renderer.get_pixel(source_x, source_y);

                let Some(color) = palette.colors.get(color as usize) else { continue };

                if color.a < 255 { continue }

                draw_pixel(&mut frame_buf.pixels, width, x as usize, y as usize, Color24::from(color), depth);
            }
        }
    }


    // pub(crate) fn draw_world_pixel(&mut self, x:f32, y:f32, color:u8) {
    //     let screen_x = x - cam_rect.x;
    //     let screen_y = y - cam_rect.y;
    //     if screen_x < 0.0 || (screen_x > (RENDER_WIDTH - 1) as f32) { return }
    //     if screen_y < 0.0 || (screen_y > (RENDER_HEIGHT - 1) as f32) { return }
    //     self.framebuf.draw_pixel(
    //         screen_x as usize,
    //         screen_y as usize,
    //         color
    //     )
    // }

}