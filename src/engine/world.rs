use slotmap::SlotMap;

use crate::*;
use core::mem::variant_count;

/// A World contains all necessary data to render and detect collisions on entities, including the
/// tile Renderer and associated data like Tilemaps and Animations.
pub struct World<
    S: Specs,
    TilesetEnum: Into<u8> + Into<usize> + Copy,
    PaletteEnum: Into<u8> + Into<usize> + Copy,
> where
    [(); variant_count::<TilesetEnum>()]: Sized,
    [(); variant_count::<PaletteEnum>()]: Sized,
    [(); S::ATLAS_WIDTH * S::ATLAS_HEIGHT]: Sized,
    [(); S::ANIMS_PER_TILESET]: Sized,
    [(); S::FONTS_PER_TILESET]: Sized,
    [(); S::TILEMAPS_PER_TILESET]: Sized,
    [(); S::COLORS_PER_PALETTE]: Sized,
    [(); S::MAX_LOADED_TILESETS]: Sized,
    [(); S::MAX_LOADED_FONTS]: Sized,
    [(); S::MAX_LOADED_ANIMS]: Sized,
    [(); S::MAX_LOADED_TILEMAPS]: Sized,
    [(); S::RENDER_WIDTH * S::RENDER_HEIGHT]: Sized,
    [(); 256 * (S::TILE_WIDTH as usize) * (S::TILE_HEIGHT as usize)]: Sized,
    [(); (S::ATLAS_WIDTH * S::ATLAS_HEIGHT) / (S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)]:
        Sized,
{
    // Visible to Host App
    pub limit_frame_rate: Option<f32>,
    pub debug_colliders: bool,
    pub debug_pivot: bool,
    pub debug_atlas: bool,
    pub draw_sprites: bool,
    pub draw_tilemaps: bool,
    pub cam: Rect<f32>,
    // Main components
    pub framebuf: FrameBuf<S>,
    pub render: Renderer<S, TilesetEnum, PaletteEnum>,

    // Private
    time_elapsed_buffer: SmoothBuffer<15>, // Affects gameplay speed (used to calculate frame deltas)
    time_update_buffer: SmoothBuffer<120>, // For performance info only, doesn't affect gameplay
    time: f32,
    time_update: f32,
    time_elapsed: f32,
    time_idle: f32,

    // Data Pools
    entities:SlotMap<EntityID, Entity>,    // Stores just the layer where each entity is
    // layers: SlotMap<LayerID, Layer>,        // Each layer contains the actual entity
    // layers: LayerPool,
}

impl<
        S: Specs,
        TilesetEnum: Into<u8> + Into<usize> + Copy,
        PaletteEnum: Into<u8> + Into<usize> + Copy,
    > World<S, TilesetEnum, PaletteEnum>
where
    [(); variant_count::<TilesetEnum>()]: Sized,
    [(); variant_count::<PaletteEnum>()]: Sized,
    [(); S::ATLAS_WIDTH * S::ATLAS_HEIGHT]: Sized,
    [(); S::ANIMS_PER_TILESET]: Sized,
    [(); S::FONTS_PER_TILESET]: Sized,
    [(); S::TILEMAPS_PER_TILESET]: Sized,
    [(); S::COLORS_PER_PALETTE]: Sized,
    [(); S::MAX_LOADED_TILESETS]: Sized,
    [(); S::MAX_LOADED_FONTS]: Sized,
    [(); S::MAX_LOADED_ANIMS]: Sized,
    [(); S::MAX_LOADED_TILEMAPS]: Sized,
    [(); S::RENDER_WIDTH * S::RENDER_HEIGHT]: Sized,
    [(); 256 * (S::TILE_WIDTH as usize) * (S::TILE_HEIGHT as usize)]: Sized,
    [(); (S::ATLAS_WIDTH * S::ATLAS_HEIGHT) / (S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)]:
        Sized,
{
    pub fn new() -> Self {
        World {
            limit_frame_rate: None,
            debug_colliders: false,
            debug_pivot: false,
            debug_atlas: false,
            draw_sprites: true,
            draw_tilemaps: true,

            cam: Rect::new(0.0, 0.0, S::RENDER_WIDTH as f32, S::RENDER_HEIGHT as f32),
            framebuf: FrameBuf::new(),
            render: Renderer::new(),

            time_elapsed_buffer: SmoothBuffer::new(),
            time_update_buffer: SmoothBuffer::new(),
            // time_internal_update_buffer: SmoothBuffer::new(),
            time: 0.0,
            time_update: 1.0 / 60.0,

            time_elapsed: 1.0 / 60.0,
            time_idle: 0.0,

            entities: Default::default(), // entities: SlotMap::with_capacity_and_key(64),
        }
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn time_elapsed(&self) -> f32 {
        self.time_elapsed
    }

    pub fn time_update(&self) -> f32 {
        self.time_update
    }

    pub fn time_idle(&self) -> f32 {
        self.time_idle
    }

    pub fn center_camera_on(&mut self, id: EntityID) {
        let Some(ent) = self.get_entity(id) else {
            return;
        };
        let pos = ent.pos;
        self.cam.x = pos.x - (self.framebuf.width() / 2) as f32;
        self.cam.y = pos.y - (self.framebuf.height() / 2) as f32;
    }

    pub fn set_viewport(&mut self, rect: Rect<i32>) {
        self.framebuf.viewport = rect;
        self.cam.w = rect.w as f32;
        self.cam.h = rect.h as f32;
    }

    // pub fn insert_layer(&mut self, palette:impl IntoPrimitive) -> LayerID {
    // pub fn insert_layer(&mut self) -> LayerID {
    //     self.layers.insert_layer()
    // }

    // Returns a reference to the entity right away so you can easily edit its fields
    // pub fn insert_entity(&mut self, layer_id: LayerID) -> &mut Entity {
    //     self.layers.insert_entity(layer_id)
    // }

    pub fn insert_entity(&mut self, depth:u8) -> EntityID {
        self.entities.insert_with_key(|key|{
            Entity::new(key, depth)
        })
    }


    pub fn delete_entity(&mut self, id: EntityID) {
        if let Some(ent) = self.entities.get(id) {
            // Clean up BgTiles if needed.
            // Tilemap will be left "dirty" by the AnimTile entity if this is not performed
            if let Shape::BgTiles { tilemap_entity, .. } = ent.shape {
                if let Some(tilemap_ent) = self.entities.get(tilemap_entity) {
                    if let Shape::Bg {
                        tileset,
                        tilemap_id,
                    } = tilemap_ent.shape
                    {
                        let tilemap = self.render.get_tilemap_mut(tileset, tilemap_id);
                        tilemap.restore_bg_buffer(ent.id);
                        tilemap.bg_buffers.remove(id);
                    }
                }
            }
            self.entities.remove(id);
        }
    }

    // Allows "breaking" the mutable refs per field, makes it a little easier to please the borrow checker
    pub fn get_members(&mut self) -> (&mut SlotMap<EntityID, Entity>, &mut Renderer<S, TilesetEnum, PaletteEnum>) {
        (&mut self.entities, &mut self.render)
    }

    #[inline]
    pub fn get_entity(&self, id: EntityID) -> Option<&Entity> {
        self.entities.get(id)
    }

    #[inline]
    pub fn get_entity_mut(&mut self, id: EntityID) -> Option<&mut Entity> {
        self.entities.get_mut(id)
    }


    pub fn set_collider(&mut self, id:EntityID, collider:Collider) {
        self.entities[id].collider = Some(collider);
    }


    pub fn remove_collider(&mut self, id:EntityID) {
        self.entities[id].collider = None;
    }


    pub fn set_shape(&mut self, id:EntityID, shape:Shape) {
        self.entities[id].shape = shape;
    }


    pub fn move_and_collide( &mut self,entity_id: EntityID,entity_vel: &mut Vec2<f32>,other_id: EntityID,other_vel: Vec2<f32>) -> Option<Collision<f32>> {
        let other = self.entities.get(other_id).unwrap().clone();
        let entity = self.entities.get_mut(entity_id)?;

        entity.move_and_collide(entity_vel, &other, other_vel, self.time_elapsed)
    }




    // // Collides a POINT to a RECT.
    // // TODO: Collider types: point, rect, circle, tilemap; then take colliders as input
    // pub fn move_and_collide(
    //     &mut self,
    //     entity_id: EntityID,
    //     entity_vel: &mut Vec2<f32>,
    //     collider_id: EntityID,
    //     collider_vel: &Vec2<f32>,
    //     bg: EntityID,
    // ) {
    //     let mut collision: Option<Collision<f32>> = None;
    //     let mut pos = self.get_position(entity_id);
    //     // let mut rect = self.get_entity_rect_from_id(entity_id);
    //     let col_rect = self.get_entity_rect_from_id(collider_id);

    //     let prev_pos = pos;
    //     // let prev_pos = if col_rect.contains(pos.x, pos.y) {
    //     //     // Uh oh, collider is overlapping initial position, before we even move! 
    //     //     rect.deintersect(*entity_vel, &col_rect);
    //     //     let offset = self.get_entity(entity_id).unwrap().render_offset;
    //     //     rect.pos() - offset.to_f32()
    //     // } else {
    //     //     // Safe to return to this position
    //     //     pos
    //     // };

    //     // Advance pos (not source entity.pos yet)
    //     pos.x += entity_vel.x * self.time_elapsed;
    //     pos.y += entity_vel.y * self.time_elapsed;
    //     // We add PI to the angle so that it points to the direction the point came from, out of the rectangle
    //     let trajectory = Ray { origin: pos, angle: entity_vel.y.atan2(entity_vel.x) + PI };

    //     // Check Collider
    //     if col_rect.contains(pos.x, pos.y) {
    //         // Entity movement
    //         if let Some((point, normal)) = col_rect.intersect_ray(&trajectory) {
    //             collision = Some(Collision{
    //                 tile: None,
    //                 point,
    //                 normal,
    //                 collider_velocity: *collider_vel,
    //             });
    //         } else {
    //             println!("Point intersection fail! This message should never print...");
    //         }
    //     } else {
    //         // Check Tilemap
    //         let move_x = Vec2 {
    //             x: pos.x,
    //             y: prev_pos.y,
    //         };
    //         let move_y = Vec2 {
    //             x: prev_pos.x,
    //             y: pos.y,
    //         };
    //         if let Some(col) = self.tilemap_raycast(bg, prev_pos.x, prev_pos.y, move_x.x, move_x.y) {
    //             collision = Some(col);
    //         } else if let Some(col) = self.tilemap_raycast(bg, prev_pos.x, prev_pos.y, move_y.x, move_y.y){
    //             collision = Some(col);
    //         }
    //     }

    //     if let Some(col) = collision {
    //         let incoming_angle = pos.angle_between(&prev_pos);
    //         let outgoing_angle = mirror_angle(incoming_angle, col.normal);

    //         // Adjust velocity reference (will be applied on next frame)
    //         let len = entity_vel.len();
    //         entity_vel.x = len * outgoing_angle.cos();
    //         entity_vel.y = len * outgoing_angle.sin();

    //         // prevents "grabbing" the puck? TODO: Needs testing
    //         entity_vel.x += col.collider_velocity.x;
    //         entity_vel.y += col.collider_velocity.y;
        
    //         // pos = prev_pos;
    //         // pos = col.point;
    //     }

    //     // Apply position after transforms
    //     if let Some(ent) = self.layers.get_mut(entity_id) {            
    //         ent.pos = pos;
    //     };
        

    // }

    pub fn get_position(&self, id: EntityID) -> Vec2<f32> {
        self.entities[id].pos
    }


    pub fn set_position(&mut self, id: EntityID, x:f32, y:f32) {
        self.entities[id].pos = Vec2 { x, y };
    }


    pub fn set_render_offset(&mut self, id: EntityID, x:i8, y:i8) {
        self.entities[id].render_offset = Vec2 { x, y };
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
            Shape::Sprite {tileset, anim_id, ..} | Shape::BgTiles {tileset, anim_id, ..} => {
                let anim = self.render.get_anim(tileset, anim_id);
                let frame = anim.frame(self.time);
                Rect {
                    x: entity.pos.x + entity.render_offset.x as f32,
                    y: entity.pos.y + entity.render_offset.y as f32,
                    w: (frame.cols as usize * S::TILE_WIDTH as usize) as f32,
                    h: (frame.rows as usize * S::TILE_HEIGHT as usize) as f32,
                }
            }
            Shape::Bg {tileset,tilemap_id} => {
                let tilemap = &self.render.get_tilemap(tileset, tilemap_id);
                Rect {
                    x: entity.pos.x + entity.render_offset.x as f32,
                    y: entity.pos.y + entity.render_offset.y as f32,
                    w: tilemap.width(S::TILE_WIDTH) as f32,
                    h: tilemap.height(S::TILE_HEIGHT) as f32,
                }
            }
        }
    }

    pub fn get_tilemap_and_rect(&self, id: EntityID) -> Option<(&Tilemap, Rect<f32>)> {
        let tilemap_entity = self.get_entity(id)?;
        let Shape::Bg {
            tileset,
            tilemap_id,
        } = tilemap_entity.shape
        else {
            return None;
        };
        let tilemap = &self.render.get_tilemap(tileset, tilemap_id);
        Some((tilemap, self.get_entity_rect(tilemap_entity)))
    }

    pub fn tile_at(&self, x: f32, y: f32, id: EntityID) -> Option<(Tile, Rect<f32>)> {
        let (tilemap, tilemap_rect) = self.get_tilemap_and_rect(id)?;
        if !tilemap_rect.contains(x, y) {
            return None;
        };

        let col = u16::try_from((x - tilemap_rect.x) as usize / S::TILE_WIDTH as usize)
            .unwrap();
        let row = u16::try_from((y - tilemap_rect.y) as usize / S::TILE_HEIGHT as usize)
            .unwrap();

        let w = S::TILE_WIDTH as f32;
        let h = S::TILE_HEIGHT as f32;
        let tile_rect = Rect {
            x: tilemap_rect.x + (col as f32 * w),
            y: tilemap_rect.y + (row as f32 * h),
            w,
            h,
        };

        Some((tilemap.get_tile(col, row), tile_rect))
    }

    pub fn tilemap_raycast(&mut self, id: EntityID, x0: f32, y0: f32, x1: f32, y1: f32) -> Option<Collision<f32>> {
        let (tilemap, tilemap_rect) = self.get_tilemap_and_rect(id)?;
        if !tilemap_rect.contains(x0, y0) {
            return None;
        };
        if !tilemap_rect.contains(x1, y1) {
            return None;
        };

        let start_col = ((x0 - tilemap_rect.x) / S::TILE_WIDTH as f32) as i32;
        let end_col = ((x1 - tilemap_rect.x) / S::TILE_WIDTH as f32) as i32;
        let start_row = ((y0 - tilemap_rect.y) / S::TILE_HEIGHT as f32) as i32;
        let end_row = ((y1 - tilemap_rect.y) / S::TILE_HEIGHT as f32) as i32;

        let collision = tilemap.collide_with_line(start_col, start_row, end_col, end_row)?;

        let w = S::TILE_WIDTH as f32;
        let h = S::TILE_HEIGHT as f32;
        let x = (collision.1.x as f32 * w) + tilemap_rect.x;
        let y = (collision.1.y as f32 * h) + tilemap_rect.y;

        let tile_rect = Rect { x, y, w, h };
        let line_collision = tile_rect.intersect_line(Line {
            start: Vec2 { x: x0, y: y0 },
            end: Vec2 { x: x1, y: y1 },
        })?;
        // println!("Actual collision!");

        Some(Collision {
            tile: Some(collision.0),
            point: line_collision.point,
            normal: line_collision.normal,
            collider_velocity: Vec2::default(),
        })
        // #[cfg(feature = "std")]{
        //     println!("collision {},{} to {},{}: {:#?}", start_col, start_row, end_col, end_row, result);
        // }
    }

    pub fn start_frame(&mut self, time_now: f32) {
        self.time_elapsed_buffer.push(time_now - self.time);
        self.time = time_now;

        self.time_elapsed = quantize(self.time_elapsed_buffer.average(), 1.0 / 360.0);
    }

    // Fills the pixel buffer with current entities
    pub fn render_frame(&mut self) {
        // Iterate entities
        for entity in self.entities.values() {
            if !entity.visible {
                continue;
            }
            let pos = entity.pos;
            let cam_rect = Rect {
                x: self.cam.x + self.framebuf.viewport.x as f32,
                y: self.cam.y + self.framebuf.viewport.y as f32,
                w: self.framebuf.viewport.w as f32,
                h: self.framebuf.viewport.h as f32,
            };

            let tile_width = S::TILE_WIDTH;
            let tile_height = S::TILE_HEIGHT;

            // Draw entity shape
            match entity.shape {
                Shape::None => {
                    // Do nothing!
                }

                Shape::BgTiles { .. } => {
                    // Shape::BgTiles { tileset, anim_id, tilemap_entity, flip_h, flip_v } => {
                    // let Some(tilemap_entity) = layer.data.get(tilemap_entity) else { continue }; // TODO: remove this "must be in same layer" requirement
                    // let Shape::Bg { tilemap_id, .. } = tilemap_entity.shape else { continue };

                    // let world_rect = self.get_entity_rect(entity);
                    // let Some(..) = world_rect.intersect(cam_rect) else { continue };

                    // let tilemap_rect = self.get_entity_rect(tilemap_entity);
                    // // let tileset = &mut self.render.tilesets[tileset as usize];
                    // let anim = self.render.get_anim(tileset, anim_id);
                    // let frame = anim.frame(self.time);

                    // let tilemap = self.render.get_tilemap(tileset, tilemap_id);

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

                Shape::Sprite {tileset, anim_id, flip_h, .. } => {
                    if !self.draw_sprites {
                        continue;
                    }
                    // Draw tiles
                    let anim = self.render.get_anim(tileset, anim_id);
                    let frame = anim.frame(self.time);
                    let palette = &self.render.palettes[anim.palette as usize];

                    let mut draw_tile = |col: u8, row: u8| {
                        let flipped_col = if flip_h { frame.cols - 1 - col } else { col };
                        let subtile = (row * frame.cols) + flipped_col;
                        let tile = frame.get_tile(subtile);
                        let abs_tile_id =
                            self.render.get_tile(tile.index, anim.tileset as usize);

                        let tile_rect = self.render.get_rect(abs_tile_id.get());
                        let quad_rect = Rect {
                            x: pos.x + (col * 8) as f32 + entity.render_offset.x as f32,
                            y: pos.y + (row * 8) as f32 + entity.render_offset.y as f32,
                            w: tile_rect.w as f32,
                            h: tile_rect.h as f32,
                        };

                        if !cam_rect.overlaps(&quad_rect) {
                            return;
                        }
                        let screen_rect = quad_rect - cam_rect.pos();

                        Self::draw_tile(
                            &mut self.framebuf,
                            &self.render,
                            screen_rect.to_i32(),
                            abs_tile_id,
                            palette,
                            flip_h ^ tile.flipped_h(), //resulting flip is a XOR
                        );
                    };

                    for row in 0..frame.rows {
                        for col in 0..frame.cols {
                            draw_tile(col, row);
                        }
                    }
                }

                Shape::Bg { tileset, tilemap_id} => {
                    if !self.draw_tilemaps {
                        continue;
                    }
                    // let tileset = &self.render.tilesets[tileset as usize];
                    let world_rect = self.get_entity_rect(entity);
                    let tilemap = self.render.get_tilemap(tileset, tilemap_id);

                    let Some(vis_rect) = world_rect.intersect(cam_rect) else {
                        continue;
                    };
                    let palette = &self.render.palettes[tilemap.palette as usize];

                    // At least a part of tilemap is visible. Render visible tiles within it.
                    let left_col = ((vis_rect.x - world_rect.x) / tile_width as f32) as usize;
                    let mut right_col =
                        ((vis_rect.right() - world_rect.x) / tile_width as f32) as usize + 1; // +1 prevents cutting off tile too early

                    let top_col = ((vis_rect.y - world_rect.y) / tile_height as f32) as usize;
                    let mut bottom_col =
                        ((vis_rect.bottom() - world_rect.y) / tile_width as f32) as usize + 1; // +1 prevents cutting off tile too early

                    // However, those +1's up there will cause invalid coordinates when we reach the end of the tilemap, so...
                    if right_col > tilemap.cols as usize {
                        right_col -= 1
                    };
                    if bottom_col > tilemap.rows as usize {
                        bottom_col -= 1
                    };

                    // Acquire and render tiles
                    for row in top_col..bottom_col {
                        for col in left_col..right_col {
                            let tile = tilemap.get_tile(col as u16, row as u16);
                            let tile_id =
                                self.render.get_tile(tile.index, tilemap.tileset as usize);

                            let tile_rect =
                                Rect::<i32>::from(self.render.get_rect(tile.index as usize));
                            let world_tile_rect = Rect {
                                x: pos.x
                                    + (col * tile_width as usize) as f32
                                    + entity.render_offset.x as f32
                                    - cam_rect.x,
                                y: pos.y
                                    + (row * tile_height as usize) as f32
                                    + entity.render_offset.y as f32
                                    - cam_rect.y,
                                w: tile_rect.w as f32,
                                h: tile_rect.h as f32,
                            };
                            Self::draw_tile(
                                &mut self.framebuf,
                                &self.render,
                                world_tile_rect.to_i32(),
                                tile_id,
                                palette,
                                tile.flipped_h(),
                            );
                        }
                    }
                }
            }

            // Draw collider Wireframe
            #[cfg(debug_assertions)]
            if self.debug_colliders {
                if let Some(col) = &entity.collider {
                    match col.kind {
                        ColliderKind::Point =>{
                            let pos = entity.pos;
                            if cam_rect.contains(pos.x, pos.y) {
                                self.framebuf.draw_pixel(pos.x as usize, pos.y as usize, COLOR_COLLIDER);
                            }
                        },
                        ColliderKind::Rect(rect) =>{
                            let world_col = entity.world_rect(rect, false);
                            let screen_col = world_col - cam_rect.pos();
                            if cam_rect.overlaps(&world_col) {
                                self.framebuf.draw_rect(screen_col.to_i32(), COLOR_COLLIDER);
                            }
                        },
                    }
                };
            }

            // Draw pivot point
            #[cfg(debug_assertions)]
            if self.debug_pivot {
                let rect = self.get_entity_rect(entity);
                if let Some(vis_rect) = rect.intersect(cam_rect) {
                    self.framebuf
                        .draw_rect((vis_rect - cam_rect).to_i32(), COLOR_ENTITY_RECT);
                };
                if let Some(point) = self
                    .framebuf
                    .get_visible_point((pos - cam_rect.pos()).to_i32())
                {
                    self.framebuf.draw_line(
                        point.x,
                        point.y - 2,
                        point.x,
                        point.y,
                        COLOR_ENTITY_RECT,
                    );
                    self.framebuf.draw_line(
                        point.x - 1,
                        point.y - 1,
                        point.x + 1,
                        point.y - 1,
                        COLOR_ENTITY_RECT,
                    );
                }
            }
        }

        // Debug Renderer
        #[cfg(debug_assertions)]
        if self.debug_atlas {
            for partition in &self.render.partitions {
                let Some(partition) = partition else { continue };
                for tile_index in partition.tiles_start_index
                    ..partition.tiles_start_index + partition.tiles_len as u16
                {
                    let rect = self.render.get_rect(tile_index as usize);
                    let palette = &self.render.palettes[partition.debug_palette as usize];
                    self.framebuf
                        .draw_filled_rect(rect.into(), Color::green_light());
                    Self::draw_tile(
                        &mut self.framebuf,
                        &self.render,
                        rect.into(),
                        TileID(tile_index),
                        palette,
                        false,
                    );
                }
            }
        }
    }

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

    pub fn draw_text(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        tileset_id: impl Into<usize> + Copy,
        font: impl Into<u8>,
        align_right: bool,
    ) {
        let font = &self.render.get_font(tileset_id, font.into());
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

            let offset_x = if align_right {
                (S::TILE_WIDTH as usize * text.len()) as i32
            } else {
                0
            };

            let abs_tile_id = self
                .render
                .get_tile(u8::try_from(index).unwrap(), font.tileset_id as usize);

            Self::draw_tile(
                &mut self.framebuf,
                &self.render,
                Rect {
                    x: x + (i * S::TILE_WIDTH as usize) as i32 - offset_x,
                    y,
                    w: S::TILE_WIDTH as i32,
                    h: S::TILE_HEIGHT as i32,
                },
                abs_tile_id,
                self.render.get_tileset_palette(tileset_id),
                false,
            )
        }
    }

    fn draw_tile(
        frame_buf: &mut FrameBuf<S>,
        renderer: &Renderer<S, TilesetEnum, PaletteEnum>,
        world_rect: Rect<i32>,
        tile: TileID,
        palette: &Palette<S>,
        flip_h: bool,
    ) {
        let Some(visible_rect) = world_rect.intersect(frame_buf.viewport) else {
            return;
        };
        let tile_rect = renderer.get_rect(tile.get());
        let width = frame_buf.width();

        for y in visible_rect.y..visible_rect.bottom() {
            let source_y = (y - world_rect.y) as usize + tile_rect.y as usize;

            for x in visible_rect.x..visible_rect.right() {
                let source_x = if flip_h {
                    let local_x = renderer.tile_width() as usize - (x - world_rect.x) as usize - 1;
                    local_x + tile_rect.x as usize
                } else {
                    let local_x = (x - world_rect.x) as usize;
                    local_x + tile_rect.x as usize
                };
                let color = renderer.get_pixel(source_x, source_y);
                let Some(color) = palette.colors.get(color as usize) else {
                    continue;
                };
                if color.a < 255 {
                    continue;
                }
                draw_pixel(&mut frame_buf.pixels, width, x as usize, y as usize, *color);
            }
        }
    }

    // pub fn clear_layer(&mut self, layer_id: LayerID) {
    //     self.layers.clear_layer(layer_id)
    // }

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

impl<S: Specs, TilesetEnum: Into<u8> + Into<usize> + Copy, PaletteEnum: Into<u8> + Into<usize> + Copy> Default for World<S, TilesetEnum, PaletteEnum>
where
    [(); variant_count::<TilesetEnum>()]: Sized,
    [(); variant_count::<PaletteEnum>()]: Sized,
    [(); S::ATLAS_WIDTH * S::ATLAS_HEIGHT]: Sized,
    [(); S::ANIMS_PER_TILESET]: Sized,
    [(); S::FONTS_PER_TILESET]: Sized,
    [(); S::TILEMAPS_PER_TILESET]: Sized,
    [(); S::COLORS_PER_PALETTE]: Sized,
    [(); S::MAX_LOADED_TILESETS]: Sized,
    [(); S::MAX_LOADED_FONTS]: Sized,
    [(); S::MAX_LOADED_ANIMS]: Sized,
    [(); S::MAX_LOADED_TILEMAPS]: Sized,
    [(); S::RENDER_WIDTH * S::RENDER_HEIGHT]: Sized,
    [(); 256 * (S::TILE_WIDTH as usize) * (S::TILE_HEIGHT as usize)]: Sized,
    [(); (S::ATLAS_WIDTH * S::ATLAS_HEIGHT) / (S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)]:
        Sized,
{
    fn default() -> Self {
        Self::new()
    }
}

// impl<
//     S:Specs,
//     TilesetEnum:Into<u8> + Into<usize> + Copy,
//     PaletteEnum:Into<u8> + Into<usize> + Copy
// > Default for World<S, TilesetEnum, PaletteEnum, AnimEnum>
// where
//     [(); variant_count::<TilesetEnum>()]: Sized,
//     [(); variant_count::<PaletteEnum>()]: Sized,
//     [(); S::ATLAS_WIDTH * S::ATLAS_HEIGHT]: Sized,
//     [(); S::ATLAS_TILE_COUNT]: Sized,
//     [(); S::ANIMS_PER_TILESET]: Sized,
//     [(); S::FONTS_PER_TILESET]: Sized,
//     [(); S::TILEMAPS_PER_TILESET]: Sized,
//     [(); S::COLORS_PER_PALETTE]: Sized,
//     [(); S::RENDER_WIDTH * S::RENDER_HEIGHT]: Sized
// {
//     fn default() -> Self {
//         Self::new()
//     }
// }
