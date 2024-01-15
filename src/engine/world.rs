use crate::*;

// pub type EntityPool = SlotMap<EntityID, LayerID>;
// pub type LayerPool = SlotMap<LayerID, Layer>;


/// A World contains all necessary data to render and detect collisions on entities, including the
/// tile Atlas and associated data like Tilemaps and Animations.
pub struct World <
    const ATLAS_LEN:usize,
    const ATLAS_TILE_COUNT:usize,
    const RENDER_LEN:usize,
    const ANIM_CAP:usize,
    const FONT_CAP:usize,
    const TILEMAP_CAP:usize,
    const PALETTE_CAP:usize,
    const COLORS_PER_PALETTE:usize
> {
    // Visible to Host App
    pub limit_frame_rate: Option<f32>,
    pub debug_colliders: bool,
    pub debug_pivot: bool,
    pub debug_atlas: bool,
    pub draw_sprites: bool,
    pub draw_tilemaps: bool,
    pub cam:Rect<f32>,
    // Main components
    pub renderer: Renderer<RENDER_LEN>,
    pub atlas: Atlas<ATLAS_LEN, ATLAS_TILE_COUNT, ANIM_CAP, FONT_CAP, TILEMAP_CAP, PALETTE_CAP, COLORS_PER_PALETTE>,
    
    // Private
    time_elapsed_buffer:SmoothBuffer<30>,
    time_update_buffer:SmoothBuffer<120>,
    time:f32,
    time_update:f32,
    time_elapsed:f32,
    time_idle:f32,
    
    // Data Pools
    // entities:SlotMap<EntityID, LayerID>,    // Stores just the layer where each entity is
    // layers: SlotMap<LayerID, Layer>,        // Each layer contains the actual entity
    layers: LayerPool

}

    
impl<
    const ATLAS_LEN:usize,
    const ATLAS_TILE_COUNT:usize,
    const RENDER_LEN:usize,
    const ANIM_CAP:usize,
    const FONT_CAP:usize,
    const TILEMAP_CAP:usize,
    const PALETTE_CAP:usize,
    const COLORS_PER_PALETTE:usize
> World<ATLAS_LEN, ATLAS_TILE_COUNT, RENDER_LEN, ANIM_CAP, FONT_CAP, TILEMAP_CAP, PALETTE_CAP, COLORS_PER_PALETTE>
{

    pub fn new(render_width:u16, render_height:u16, atlas_width:u16, atlas_height:u16, tile_width:u8, tile_height:u8) -> Self {
        World {
            limit_frame_rate: None,
            debug_colliders:false,
            debug_pivot: false,
            debug_atlas: false,
            draw_sprites: true,
            draw_tilemaps: true,

            cam: Rect::new(0.0, 0.0, render_width as f32, render_height as f32),
            renderer: Renderer::new(render_width, render_height),
            atlas: Atlas::new(atlas_width, atlas_height, tile_width, tile_height),

            time_elapsed_buffer: SmoothBuffer::new(),
            time_update_buffer: SmoothBuffer::new(),
            time: 0.0,
            time_update:1.0 / 60.0,
            
            time_elapsed: 1.0 / 60.0,
            time_idle: 0.0,

            layers: Default::default()
            // entities: SlotMap::with_capacity_and_key(64),
            // layers: SlotMap::with_capacity_and_key(4),
            // anims: SlotMap::with_capacity_and_key(64),
            // tilemaps: SlotMap::with_capacity_and_key(4)
        }
    }


    pub fn time(&self) -> f32 { self.time }


    pub fn time_elapsed(&self) -> f32 { self.time_elapsed }


    pub fn time_update(&self) -> f32 { self.time_update }


    pub fn time_idle(&self) -> f32 { self.time_idle }

    
    pub fn center_camera_on(&mut self, id:EntityID) {
        // let Some(layer) = &self.layers.get(layer) else { return };
        let Some(ent) = self.get_entity(id) else { return };
        let pos = ent.pos;
        self.cam.x = pos.x - (self.renderer.width()/2) as f32;
        self.cam.y = pos.y - (self.renderer.height()/2) as f32;
    }


    pub fn set_viewport(&mut self, rect:Rect<i32>) {
        self.renderer.viewport = rect;
        self.cam.w = rect.w as f32;
        self.cam.h = rect.h as f32;
    }



    // pub fn insert_layer(&mut self, palette:impl ByteID) -> LayerID {
    pub fn insert_layer(&mut self) -> LayerID {
        self.layers.insert_layer()
    }


    // Returns a reference to the entity right away so you can easily edit its fields
    pub fn insert_entity(&mut self, layer_id:LayerID) -> &mut Entity {
        self.layers.insert_entity(layer_id)
    }


    // Allows "breaking" the mutable refs per field, makes it a little easier to please the borrow checker
    pub fn get_data_mut(&mut self) -> (&mut LayerPool, &mut Pool<Anim, ANIM_CAP>, &mut Pool<Tilemap, TILEMAP_CAP>) {
        (&mut self.layers,  &mut self.atlas.anims, &mut self.atlas.tilemaps)
    }


    pub fn get_anim(&self, id:u8) -> &Anim {
        &self.atlas.anims.get(id as usize).unwrap()
    }


    #[inline]
    pub fn get_entity(&self, id:EntityID) -> Option<&Entity> {
        self.layers.get(id)
    }

    #[inline]
    pub fn get_entity_mut(&mut self, id:EntityID) -> Option<&mut Entity> {
        self.layers.get_mut(id)
    }


    pub fn get_tilemap(&self, id:u8) -> &Tilemap {
        &self.atlas.tilemaps.get(id as usize).unwrap()
    }


    pub fn get_tilemap_mut(&mut self, id:u8) -> &mut Tilemap {
        self.atlas.tilemaps.get_mut(id as usize).unwrap()
    }


    pub fn get_entity_rect_from_id(&self, id:EntityID) -> Rect<f32> {
        if let Some(entity) = self.get_entity(id){
            self.get_entity_rect(entity)
        } else {
            Rect::default()
        }
    }


    pub fn get_entity_rect(&self, entity:&Entity) -> Rect<f32> {
        match entity.shape {
            Shape::None => Default::default(),
            Shape::Sprite { anim_id, .. } | Shape::AnimTiles { anim_id, .. } => {
                let Some(anim) = &self.atlas.anims.get(anim_id as usize) else { return Default::default() };
                let frame = anim.frame(self.time);
                Rect {
                    x:entity.pos.x + entity.render_offset.x as f32,
                    y:entity.pos.y + entity.render_offset.y as f32,
                    w:(frame.cols as usize * self.atlas.tile_width() as usize) as f32,
                    h:(frame.rows as usize * self.atlas.tile_height() as usize) as f32,
                }
            },
            Shape::TilemapLayer { tilemap_id } => {
                let tilemap = &self.atlas.tilemaps.get(tilemap_id as usize).unwrap();
                let result = Rect {
                    x:entity.pos.x + entity.render_offset.x as f32,
                    y:entity.pos.y + entity.render_offset.y as f32,
                    w:tilemap.width(self.atlas.tile_width()) as f32,
                    h:tilemap.height(self.atlas.tile_height()) as f32
                };
                result
            },
        }
    }


    pub fn delete_entity(&mut self, id:EntityID) {
        if let Some(ent) = self.layers.get(id){
            // Clean up AnimTiles if needed.
            // Tilemap will be left "dirty" by the AnimTile entity if this is not performed
            if let Shape::AnimTiles { tilemap_entity, .. } = ent.shape {
                if let Some(tilemap_ent) = self.layers.get(tilemap_entity) {
                    if let Shape::TilemapLayer { tilemap_id } = tilemap_ent.shape {
                        if let Some(tilemap) = self.atlas.tilemaps.get_mut(tilemap_id as usize){
                            tilemap.restore_bg_buffer(ent.id);
                            tilemap.bg_buffers.remove(id);
                        };
                    }
                }
            }
            self.layers.remove_entity(id);
        }
    }


    pub fn tile_at(&self, x:f32, y:f32, id:EntityID) -> Option<Tile> {
        // let Some(entity) = layer.entities.get(id) else { return None };
        let Some(entity) = self.get_entity(id) else { return None };
        let Shape::TilemapLayer{ tilemap_id } = entity.shape else { return None };
        
        let tilemap = &self.atlas.tilemaps.get(tilemap_id as usize).unwrap();
        let rect = self.get_entity_rect(entity);

        if !rect.contains(x, y) { return None };

        let col = ((x - rect.x) as usize / self.atlas.tile_width() as usize) as u16;
        let row = ((y - rect.y) as usize / self.atlas.tile_height() as usize) as u16;

        Some(tilemap.get_tile(col, row))
    }


    // Fills the pixel buffer with current entities
    pub fn render_frame(&mut self){
        
        // Iterate entities
        for layer in self.layers.layers.values() {
            for entity in layer.data.values() {
                if !entity.visible { continue }
                if let Shape::None = entity.shape { continue }
                let pos = entity.pos;
                let cam_rect = Rect {
                    x: self.cam.x + self.renderer.viewport.x as f32,
                    y: self.cam.y + self.renderer.viewport.y as f32,
                    w: self.renderer.viewport.w as f32,
                    h: self.renderer.viewport.h as f32,
                };
    
                let tile_width = self.atlas.tile_width();
                let tile_height = self.atlas.tile_height();
    
                // Draw entity shape
                match entity.shape {
                    Shape::None => {
                        // Do nothing!
                    },
    
                    Shape::AnimTiles { anim_id, tilemap_entity, flip_h, flip_v } => {
                        let Some(tilemap_entity) = layer.data.get(tilemap_entity) else { continue }; // TODO: remove this "must be in same layer" requirement
                        let Shape::TilemapLayer { tilemap_id } = tilemap_entity.shape else { continue };
                        
                        let world_rect = self.get_entity_rect(entity);
                        let Some(..) = world_rect.intersect(cam_rect) else { continue };
                        
                        let Some(anim) = self.atlas.anims.get(anim_id as usize) else { continue };
                        let frame = anim.frame(self.time);
                        
                        let tilemap_rect = self.get_entity_rect(tilemap_entity);
                        
                        let tilemap = &mut self.atlas.tilemaps.get_mut(tilemap_id as usize).unwrap();
                        
                        let left_col = (world_rect.x - tilemap_rect.x) as i32 / tile_width as i32;
                        let top_row = (world_rect.y - tilemap_rect.y) as i32 / tile_height as i32;
    
                        if tilemap.store_bg_buffer(left_col, top_row, frame.cols, frame.rows, entity.id) == false {
                            continue
                        };
    
                        for row in 0 .. frame.rows as i32 {
                            if row < 0 { continue }
                            for col in 0 .. frame.cols as i32 {
                                if col < 0 { continue }
                                let mut tile = frame.get_tile(row as u8 * frame.cols + col as u8);
                                tile.set_flipped_h(tile.flipped_h() ^ flip_h);  //TODO: flipping needs testing
                                tile.set_flipped_v(tile.flipped_v() ^ flip_v);  //TODO: flipping needs testing
                                let tilemap_index = (((row + top_row) * tilemap.cols as i32) + (col + left_col)) as usize;
                                tilemap.tiles[tilemap_index] = tile;
                            }
                        }
                    },
    
                    Shape::Sprite { anim_id, flip_h, .. } => {
                        if !self.draw_sprites { continue }
                        // Draw tiles
                        let Some(anim) = &self.atlas.anims.get(anim_id as usize) else { continue };
                        let frame = anim.frame(self.time);
    
                        let mut draw_tile = |col:u8, row:u8| {
                            let flipped_col = if flip_h { frame.cols - 1 - col } else { col };
                            let subtile = (row*frame.cols) + flipped_col;
                            let tile = frame.get_tile(subtile);
                            let abs_tile_id = self.atlas.get_tile_from_tileset(
                                tile.index,
                                anim.tileset
                            );
    
                            let tile_rect = self.atlas.get_rect(abs_tile_id.get());
                            let quad_rect = Rect{
                                x: pos.x + (col * 8) as f32 + entity.render_offset.x as f32,
                                y: pos.y + (row * 8) as f32 + entity.render_offset.y as f32,
                                w: tile_rect.w as f32,
                                h: tile_rect.h as f32
                            };
    
                            if !cam_rect.overlaps(&quad_rect) { return }
                            let screen_rect = quad_rect - cam_rect.pos();
                            Self::draw_tile(
                                &mut self.renderer,
                                &self.atlas,
                                screen_rect.to_i32(),
                                abs_tile_id,
                                flip_h ^ tile.flipped_h() //resulting flip is a XOR
                            ); 
                        };
    
                        for row in 0 .. frame.rows {
                            for col in 0 .. frame.cols {
                                draw_tile(col, row);
                            }
                        }
                    },
    
                    Shape::TilemapLayer { tilemap_id } => {
                        if !self.draw_tilemaps { continue }
                        let world_rect = self.get_entity_rect(entity);
                        let tilemap = &mut self.atlas.tilemaps.get(tilemap_id as usize).unwrap();
                        let Some(vis_rect) = world_rect.intersect(cam_rect) else { continue };  
    
                        // At least a part of tilemap is visible. Render visible tiles within it.
                        let left_col = ((vis_rect.x - world_rect.x) / tile_width as f32) as usize;
                        let mut right_col = ((vis_rect.right() - world_rect.x) / tile_width as f32) as usize + 1; // +1 prevents cutting off tile too early
    
                        let top_col = ((vis_rect.y - world_rect.y) / tile_height as f32) as usize;
                        let mut bottom_col = ((vis_rect.bottom() - world_rect.y) / tile_width as f32) as usize + 1; // +1 prevents cutting off tile too early
    
                        // However, those +1's up there will cause invalid coordinates when we reach the end of the tilemap, so...
                        if right_col > tilemap.cols as usize { right_col -= 1 };
                        if bottom_col > tilemap.rows as usize { bottom_col -= 1 };
                        
                        // Acquire and render tiles
                        for row in top_col .. bottom_col {
                            for col in left_col .. right_col {
                                let tile = tilemap.get_tile(col as u16, row as u16);
                                let tile_id = self.atlas.get_tile_from_tileset(tile.index, tilemap.tileset);
    
                                let tile_rect = Rect::<i32>::from(self.atlas.get_rect(tile.index as usize));
                                let world_tile_rect = Rect{
                                    x: pos.x + (col * tile_width as usize) as f32 + entity.render_offset.x as f32 - cam_rect.x,
                                    y: pos.y + (row * tile_height as usize) as f32 + entity.render_offset.y as f32 - cam_rect.y,
                                    w: tile_rect.w as f32,
                                    h: tile_rect.h as f32
                                };
                                Self::draw_tile(
                                    &mut self.renderer,
                                    &self.atlas,
                                    world_tile_rect.to_i32(), tile_id, tile.flipped_h()
                                );
                            }
                        }
                    }
                }
    
                // Draw collider Wireframe
                // #[cfg(debug_assertions)]
                if self.debug_colliders {
                    if let Some(col) = entity.col {
                        let world_col = entity.world_rect(col, false);
                        let screen_col = world_col - cam_rect.pos();
                        if cam_rect.overlaps(&world_col) {
                            self.renderer.draw_rect(screen_col.to_i32(), COLOR_COLLIDER);
                        }
                    }; 
                }
    
                // Draw pivot point
                // #[cfg(debug_assertions)]
                if self.debug_pivot {
                    let rect = self.get_entity_rect(entity);
                    if let Some(vis_rect) = rect.intersect(cam_rect) {
                        self.renderer.draw_rect((vis_rect - cam_rect).to_i32(), COLOR_ENTITY_RECT);
                    };
                    if let Some(point) = self.renderer.get_visible_point((pos - cam_rect.pos()).to_i32()){
                        self.renderer.draw_line(point.x, point.y-2, point.x, point.y, 15);
                        self.renderer.draw_line(point.x-1, point.y-1, point.x+1, point.y-1, 15);
                    }
                }
            }
    
            // Debug Atlas
            // #[cfg(debug_assertions)]
            if self.debug_atlas {
                'draw_loop: {
                    let width = self.atlas.width();
                    for y in 0 .. self.atlas.height() {
                        for x in 0 .. width  {
                            let pixel_index = (y*self.renderer.width()) + x;
                            if pixel_index > RENDER_LEN - 1 { break 'draw_loop }
                            self.renderer.pixels[pixel_index] = self.atlas.get_pixel(x, y);
                        }
                    }
                }
            }
        }
    }


    pub fn draw_text(&mut self, text:&str, x:i32, y:i32, font:impl ByteID, align_right:bool) {
        let font:u8 = font.to_u8();
        let Some(font_range) = &self.atlas.fonts.get(font.into()) else { return }; // TODO: This SHOULD fail if no fonts
        let tileset_start = self.atlas.get_tileset(font_range.tileset).start_index;
        for (i,letter) in text.chars().enumerate() {
            let letter = letter as u32;
            let index = if letter > 47 {
                if letter < 65 {
                    (letter - 48) as u16 + font_range.start as u16 // Zero
                } else {
                    (letter - 55) as u16 + font_range.start as u16 // Upper Case 'A' (A index is 65, but the first 10 tiles are the numbers so we add 10)
                }
            } else {
                font_range.last() as u16 // Space
            };
            
            let offset_x = if align_right {
                (self.atlas.tile_width() as usize * text.len()) as i32
            } else {
                0
            };

            Self::draw_tile(
                &mut self.renderer,
                &self.atlas,
                Rect {
                    x: x + (i * self.atlas.tile_width() as usize) as i32 - offset_x,
                    y,
                    w: self.atlas.tile_width() as i32,
                    h: self.atlas.tile_height() as i32
                },
                TileID(index + tileset_start),
                false
            )
        }
    }


    fn draw_tile(
        renderer: &mut Renderer<RENDER_LEN>,
        atlas:&Atlas<ATLAS_LEN, ATLAS_TILE_COUNT, ANIM_CAP, FONT_CAP, TILEMAP_CAP, PALETTE_CAP, COLORS_PER_PALETTE>,
        world_rect:Rect<i32>,
        tile:TileID,
        flip_h:bool
    ){
        let Some(visible_rect) = world_rect.intersect(renderer.viewport) else { return };
        let tile_rect = atlas.get_rect(tile.get());
        let width = renderer.width();
        
        for y in visible_rect.y .. visible_rect.bottom() {
            let source_y = (y - world_rect.y) as usize + tile_rect.y as usize;

            for x in visible_rect.x .. visible_rect.right() {
                let source_x = if flip_h {
                    let local_x = atlas.tile_width() as usize - (x - world_rect.x) as usize - 1;
                    local_x + tile_rect.x as usize
                } else {    
                    let local_x = (x - world_rect.x) as usize;
                    local_x + tile_rect.x as usize
                };
                let color = atlas.get_pixel(source_x, source_y);
                if color == COLOR_TRANSPARENCY { continue; } 
                draw_pixel(&mut renderer.pixels, width, x as usize, y as usize, color);
            }
        }
    }


    // pub(crate) fn draw_world_pixel(&mut self, x:f32, y:f32, color:u8) {
    //     let screen_x = x - cam_rect.x;
    //     let screen_y = y - cam_rect.y;
    //     if screen_x < 0.0 || (screen_x > (RENDER_WIDTH - 1) as f32) { return }
    //     if screen_y < 0.0 || (screen_y > (RENDER_HEIGHT - 1) as f32) { return }
    //     self.renderer.draw_pixel(
    //         screen_x as usize,
    //         screen_y as usize,
    //         color
    //     )
    // }


    pub fn start_frame(&mut self, time_now:f32) {
        self.time_elapsed_buffer.push(time_now - self.time);
        self.time = time_now;
        
        self.time_elapsed = quantize(self.time_elapsed_buffer.average(), 1.0/ 360.0);
    }



    pub fn finish_frame(&mut self, time_now:f32) {
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

    
}




