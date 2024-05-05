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


    pub(crate) fn draw_collider(framebuf:&mut FrameBuf, cam_rect:&Rect<i32>, probe:&CollisionProbe<f32>, color:Color32){
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

            let abs_tile_id = self.renderer.get_tile(
                u8::try_from(index).unwrap(), font.tileset_id
            );

            Self::draw_tile(
                &mut self.framebuf,
                &self.renderer,
                self.renderer.get_tileset_palette(info.tileset_id),
                Rect {
                    x: x + (i * self.specs.tile_width as usize) as i32 - offset_x,
                    y,
                    w: self.specs.tile_width as i32,
                    h: self.specs.tile_height as i32,
                },
                TileInfo {
                    tile: abs_tile_id,
                    flip_h: false,
                    flip_v: false,
                    depth: info.depth,
                },
            )
        }
    }


    pub(crate) fn draw_tile(
        frame_buf: &mut FrameBuf,
        renderer: &Renderer<T,P>,
        palette: &Palette,
        world_rect: Rect<i32>,
        tile_info: TileInfo,
    ) {
        let Some(visible_rect) = world_rect.intersect(frame_buf.viewport) else { return };
        let width = frame_buf.width();
        let tile_width = renderer.tile_width() as usize;
        let tile_height = renderer.tile_height() as usize;
        let tile_len = tile_width * renderer.tile_height() as usize;
        let source_index = tile_info.tile.get() * tile_len;
        
        for y in visible_rect.y .. visible_rect.bottom() {
            for x in visible_rect.x .. visible_rect.right() {

                if x < 0 || y < 0 { continue }

                // Get x coordinate within a tile 
                let local_x = if tile_info.flip_h {
                    tile_width - (x - world_rect.x) as usize - 1
                } else {
                    (x - world_rect.x) as usize
                };
                
                // Get y coordinate within a tile
                let local_y = if tile_info.flip_v {
                    tile_height - (y - world_rect.y) as usize - 1
                } else {
                    (y - world_rect.y) as usize
                };
                
                // // Get source color
                let tile_pixel_index = local_x + (local_y * tile_width);
                let source_color = renderer.tile_pixels.data[source_index + tile_pixel_index] as usize;
                let Some(color) = palette.colors.get(source_color) else { continue };

                draw_pixel(&mut frame_buf.pixels, width, x as usize, y as usize, *color, tile_info.depth);
            }
        }
    }

}