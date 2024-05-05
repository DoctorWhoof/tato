use num_traits::ToPrimitive;

use crate::*;

impl<T,P> World<T,P>
where T:TilesetEnum, P:PaletteEnum {

// **************************************** Events ***************************************


    /// Sets up current frame's timing 
    pub fn start_frame(&mut self, time_now: f32) {
        self.time_elapsed_buffer.push(time_now - self.time);
        self.time = time_now;
        self.time_elapsed = quantize(self.time_elapsed_buffer.average(), 1.0 / 360.0);
    }


    /// Wraps up current frame's timing
    pub fn finish_frame(&mut self, time_now: f32) {
        self.time_update_buffer.push(time_now - self.time);
        self.time_update = self.time_update_buffer.average();

        // Limit frame rate. TODO: This is hacky, doesn't always work, and needs std library.
        // if let Some(fps_limit) = self.limit_frame_rate {
        //     let immediate_fps = 1.0 / self.time_update;
        //     if immediate_fps > fps_limit {
        //         let time_target = 1.0 / fps_limit;
        //         let time_diff = time_target - self.time_update;
        //         if time_diff > 1.0 / 240.0 {
        //             self.time_idle = time_diff * 0.75;
        //             sleep(Duration::from_secs_f32(self.time_idle));
        //         }
        //     } else {
        //         println!("Skipping idle cycle!");
        //         self.time_idle = 0.0;
        //     }
        // } else {
        //     self.time_idle = 0.0;
        // }
    }


    /// Fills the pixel buffer with current entities
    pub fn render_frame(&mut self) {
        // Iterate entities
        let cam_rect = Rect {
            x: self.cam.x.floor() as i32,
            y: self.cam.y.floor() as i32,
            // If I take the viewport size into account here it actually doesn't render tilemaps correctly!
            w: self.specs.render_width as i32,
            h: self.specs.render_height as i32
        };
        
        let tile_width = self.specs.tile_width as i32;
        let tile_height = self.specs.tile_height as i32;

        for entity in self.entities.values() {
            // Draw entity shape
            if !entity.visible { continue }

            let pos = entity.pos.to_i32();

            match entity.shape {
                Shape::None => {
                    // Do nothing!
                }

                Shape::BgSprite { .. } => {
                    // Shape::BgTiles { tileset, anim_id, tilemap_entity, flip_h, flip_v } => {
                    // let Some(tilemap_entity) = layer.data.get(tilemap_entity) else { continue }; // TODO: remove this "must be in same layer" requirement
                    // let Shape::Bg { tilemap_id, .. } = tilemap_entity.shape else { continue };

                    // let world_rect = self.get_entity_rect(entity);
                    // let Some(..) = world_rect.intersect(cam_rect) else { continue };

                    // let tilemap_rect = self.get_entity_rect(tilemap_entity);
                    // // let tileset = &mut self.renderer.tilesets[tileset as usize];
                    // let anim = self.renderer.get_anim(tileset, anim_id);
                    // let frame = anim.frame(self.time);

                    // let tilemap = self.renderer.get_tilemap(tileset, tilemap_id);

                    // let left_col = (world_rect.x - tilemap_rect.x) as i32 / tile_width as i32;
                    // let top_row = (world_rect.y - tilemap_rect.y) as i32 / tile_height as i32;

                    // if !tilemap.store_bg_buffer(left_col, top_row, frame.cols, frame.rows, entity.id) {
                    //     continue
                    // };

                    // for row in 0 .. frame.rows as i32 {
                    //     if row < 0 { continue }
                    //     for col in 0 .. frame.cols as i32 {
                    //         if col < 0 { continue }
                    //         let mut tile = frame.get_tile(row as u8 * frame.cols + col as u8);
                    //         tile.set_flipped_h(tile.flipped_h() ^ flip_h);  //TODO: flipping needs testing
                    //         tile.set_flipped_v(tile.flipped_v() ^ flip_v);  //TODO: flipping needs testing
                    //         let tilemap_index = (((row + top_row) * tilemap.cols as i32) + (col + left_col)) as usize;
                    //         tilemap.tiles[tilemap_index] = tile;
                    //     }
                    // }
                }

                Shape::Sprite {tileset_id, anim_id, flip_h, flip_v } => {
                    if !self.draw_sprites { continue }
                    // Draw tiles
                    let anim = self.renderer.get_anim(tileset_id, anim_id);
                    let frame = anim.frame(self.time);

                    let Some(palette) = &self.renderer.palettes[anim.palette as usize] else { return };

                    let mut draw_tile = |col: u8, row: u8| {
                        let flipped_col = if flip_h { frame.cols - 1 - col } else { col };
                        let subtile = (row * frame.cols) + flipped_col;
                        let tile = frame.get_tile(subtile);
                        let abs_tile_id = self.renderer.get_tile(tile.index, anim.tileset);

                        let quad_rect = Rect {
                            x: pos.x + (col as i32 * tile_width)  + entity.render_offset.x as i32,
                            y: pos.y + (row as i32 * tile_height) + entity.render_offset.y as i32,
                            w: tile_width,
                            h: tile_height,
                        };

                        let screen_rect = quad_rect - cam_rect.pos();

                        Self::draw_tile(
                            &mut self.framebuf,
                            &self.renderer,
                            palette,
                            screen_rect,
                            TileInfo {
                                tile: abs_tile_id,
                                flip_h: flip_h ^ tile.flipped_h(), //resulting flip is a XOR
                                flip_v: flip_v ^ tile.flipped_v(),
                                depth: entity.depth
                            },
                        );
                    };

                    for row in 0..frame.rows {
                        for col in 0..frame.cols {
                            draw_tile(col, row);
                        }
                    }
                }

                Shape::Bg { tileset_id, tilemap_id } => {
                    if !self.draw_tilemaps {
                        continue;
                    }
                    let world_rect = self.get_entity_rect(entity).to_i32();
                    let tilemap = self.renderer.get_tilemap(tileset_id, tilemap_id);

                    let Some(vis_rect) = world_rect.intersect(cam_rect) else { continue };

                    let Some(palette) = &self.renderer.palettes[tilemap.palette as usize] else { continue };

                    // At least a part of tilemap is visible. Render visible tiles within it.
                    // +1 prevents cutting off tiles too early at the edges.
                    let Some(left_col) = ((vis_rect.x - world_rect.x) / tile_width).to_u16()
                        else { continue };
                    let Some(right_col) = (((vis_rect.right() - world_rect.x) / tile_width) + 1).to_u16()
                        else { continue };

                    let Some(top_col) = ((vis_rect.y - world_rect.y) / tile_height).to_u16()
                        else { continue };
                    let Some(bottom_col) = (((vis_rect.bottom() - world_rect.y) / tile_width) + 1).to_u16()
                        else { continue }; 

                    // Acquire and render tiles
                    for row in top_col .. bottom_col {
                        for col in left_col .. right_col {
                            let Some(tile) = tilemap.get_tile(col as u16, row as u16) else { continue };
                            let tile_id = self.renderer.get_tile(tile.index, tilemap.tileset);
                            let tile_rect = Rect::<i32>::from(self.renderer.get_tile_rect(tile.index));
                            
                            let world_tile_rect = Rect {
                                x: pos.x
                                    + (col as i32 * tile_width)
                                    + entity.render_offset.x as i32
                                    - cam_rect.x,
                                y: pos.y
                                    + (row as i32 * tile_height)
                                    + entity.render_offset.y as i32
                                    - cam_rect.y,
                                w: tile_rect.w,
                                h: tile_rect.h,
                            };

                            Self::draw_tile (
                                &mut self.framebuf,
                                &self.renderer,
                                palette,
                                world_tile_rect,
                                TileInfo {
                                    tile: tile_id,
                                    flip_h: tile.flipped_h(),
                                    flip_v: tile.flipped_v(),
                                    depth: entity.depth
                                },
                            );
                        }
                    }
                }
            }

            // Draw pivot point
            #[cfg(debug_assertions)]
            if self.debug_wireframe {
                let rect = self.get_entity_rect(entity).to_i32();
                if let Some(vis_rect) = rect.intersect(cam_rect) {
                    self.framebuf
                        .draw_rect(vis_rect - cam_rect, COLOR_ENTITY_RECT);
                };
                if let Some(point) = self
                    .framebuf
                    .get_visible_point(pos - cam_rect.pos())
                {
                    self.framebuf.draw_line(
                        point.x,
                        point.y - 2,
                        point.x,
                        point.y,
                        COLOR_ENTITY_RECT,
                        255
                    );
                    self.framebuf.draw_line(
                        point.x - 1,
                        point.y - 1,
                        point.x + 1,
                        point.y - 1,
                        COLOR_ENTITY_RECT,
                        255
                    );
                }
            }
        }

        // Debug Renderer
        #[cfg(debug_assertions)]
        if self.debug_atlas {
            // let width = self.framebuf.width();
            // for (tileset_id, block) in self.renderer.tile_pixels.blocks.iter().enumerate() {
            //     if let Some(block) = block {
            //         for index in block.start .. block.start + block.length {
            //             let Some(palette) = &self.renderer.palettes[tileset_id] else { return };
            //             let source_pix = self.renderer.tile_pixels.data[index];
            //             let color = palette.colors[source_pix as usize];
            //             let x = index % self.renderer.width() as usize;
            //             let y = index / self.renderer.width() as usize;
            //             draw_pixel(&mut self.framebuf.pixels, width, x, y, color, 255);
            //         }   
            //     }
            // }
            for (tileset_id, block) in self.renderer.tile_indices.blocks.iter().enumerate() {
                if let Some(block) = block {
                    for index in block.start .. block.start + block.length {
                        let rect = self.renderer.get_tile_rect(index);
                        let Some(palette) = &self.renderer.palettes[tileset_id] else { return };
                        self.framebuf
                            .draw_filled_rect(rect.into(), Color32::green_light());
                        Self::draw_tile(
                            &mut self.framebuf,
                            &self.renderer,
                            palette,
                            rect.into(),
                            TileInfo {
                                tile: TileID(index as u16),
                                flip_h: false,
                                flip_v: false,
                                depth: 255
                            },
                        );
                    }   
                }
            }
        }

        // Draw collider Wireframe
        #[cfg(debug_assertions)]
        if self.debug_colliders {
            // Colliders
            for layer in &self.collision_layers {
                for probe in layer.values() {
                    // let Some(probe) = probe else { continue };
                    Self::draw_collider(&mut self.framebuf, &cam_rect, probe, COLOR_COLLIDER);
                }
            }
        }
    }


}