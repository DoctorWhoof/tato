use core::time::Duration;
use std::{time::Instant, thread::sleep};
use crate::*;

pub const ANIM_CAPACITY:u16 = 256;
pub const ENTITY_CAPACITY:u16 = 256;
pub const TILEMAP_CAPACITY:u16 = 1;


macro_rules! generate_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name(pub u16);

        impl $name {
            #[allow(unused)] #[inline]
            pub fn get(self) -> usize { self.0 as usize}
        }
    }   
}

generate_id!(EntityID);
generate_id!(AnimID);


pub struct World {
    // Visible to Host App
    pub limit_frame_rate: Option<f32>,
    pub debug_colliders: bool,
    pub debug_pivot: bool,
    pub debug_atlas: bool,
    pub draw_sprites: bool,
    pub draw_tilemaps: bool,

    pub cam:Rect<f32>,
    pub renderer: Renderer,
    pub time:f32,
    pub elapsed_time_buffer:SmoothBuffer,
    pub time_update_buffer:SmoothBuffer,
    
    // Private
    entity_head:u16,
    anim_head:u16,
    tilemap_head:u16,
    time_start:Instant,
    time_start_frame:Instant,
    time_update:f32,
    time_elapsed:f32,
    time_idle:f32,
    
    // Main data
    anims:[Anim; ANIM_CAPACITY as usize],
    entities:[Entity;ENTITY_CAPACITY as usize],
    tilemaps:[Tilemap; TILEMAP_CAPACITY as usize],
}


impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
    
impl World {


    pub fn new() -> Self {
        let renderer = Renderer::default();
        World {
            limit_frame_rate: None,
            debug_colliders:false,
            debug_pivot: false,
            debug_atlas: false,
            draw_sprites: true,
            draw_tilemaps: true,

            entity_head:0,
            anim_head:0,
            tilemap_head:0,
            time_start:Instant::now(),
            time_start_frame:Instant::now(),
            time_update:1.0 / 60.0,
            time_elapsed: 1.0 / 60.0,
            time_idle: 0.0,

            cam: Rect::new(0.0, 0.0, RENDER_WIDTH as f32, (RENDER_HEIGHT - HUD_HEIGHT) as f32),
            entities: core::array::from_fn(|_| Entity::default() ),
            anims: core::array::from_fn(|_| Anim::default() ),
            tilemaps: core::array::from_fn(|_| Tilemap::default() ),
            renderer,
            
            time: 0.0,
            elapsed_time_buffer: SmoothBuffer::new(),
            time_update_buffer: SmoothBuffer::new()
        }
    }


    pub fn time_elapsed(&self) -> f32 { self.time_elapsed }


    pub fn time_update(&self) -> f32 { self.time_update }


    pub fn time_idle(&self) -> f32 { self.time_idle }

    
    pub fn center_camera_on(&mut self, entity_id:EntityID) {
        let e = &self.entities[entity_id.get()];
        self.cam.x = e.pos.x - (RENDER_WIDTH/2) as f32;
        self.cam.y = e.pos.y - (RENDER_HEIGHT/2) as f32;
    }


    // Returns a reference to the entity right away so you can easily edit its fields
    pub fn insert_entity(&mut self) -> &mut Entity {
        let id = self.entity_head;
        if id == ENTITY_CAPACITY { panic!("World: Entity capacity exceeded.") }

        self.entities[id as usize].id = EntityID(id);

        self.entity_head += 1;
        &mut self.entities[id as usize]
    }

    // // Returns a reference to the tilemap just inserted.
    // pub(crate) fn insert_tilemap(&mut self, cols:usize, rows:usize) -> &mut Tilemap {
    //     let id = self.tilemap_head;
    //     if id == TILEMAP_CAPACITY { panic!("World: Tilemap capacity exceeded.") }
    //     if cols * rows > TILEMAP_LEN {
    //         panic!("Tilemap: Error creating {} x {} tilemap, capacity of {} exceeded", cols, rows, TILEMAP_LEN)
    //     }
    //     self.tilemaps[id as usize] = Tilemap::default();
    //     self.entity_head += 1;
    //     &mut self.tilemaps[id as usize]
    // }


    // Returns a reference to the tilemap just inserted.
    pub fn load_tilemap(&mut self, data:&[u8], tileset:TilesetID) -> TilemapID {
        // Create ID and perform basic checks
        let id = self.tilemap_head;
        if id == TILEMAP_CAPACITY { panic!("World: Tilemap capacity exceeded.") }
        
        // Create tilemap
        self.tilemaps[id as usize] = Tilemap::load( data, tileset, TilemapID(id) );
        self.entity_head += 1;
        self.tilemaps[id as usize].id
    }


    pub fn insert_anim(&mut self, bytes:&[u8], tileset:TilesetID, fps:u8) -> AnimID {
        let result = self.anim_head;
        let anim = Anim::load(bytes, tileset, fps);

        self.anims[self.anim_head as usize] = anim;
        self.anim_head += 1;
        AnimID(result)
    }


    // Allows "breaking" the mutable refs per field, makes it a little easier to please the borrow checker
    pub fn get_data_mut(&mut self) -> (&mut[Entity], &mut[Anim], &mut[Tilemap]) {
        (&mut self.entities, &mut self.anims, &mut self.tilemaps)
    }


    pub fn get_entity(&self, id:EntityID) -> &Entity { &self.entities[id.get()] }


    // pub fn get_entity_mut(&mut self, id:EntityID) -> &mut Entity { &mut self.entities[id.get()] }


    pub fn get_tilemap(&self, id:TilemapID) -> &Tilemap { &self.tilemaps[id.get()] }


    // pub fn get_tilemap_mut(&mut self, id:TilemapID) -> &mut Tilemap { &mut self.tilemaps[id.get()] }


    pub fn get_entity_rect_from_id(&self, id:EntityID) -> Rect<f32> {
        let entity = &self.entities[id.get()];
        self.get_entity_rect(entity)
    }


    pub fn get_entity_rect(&self, entity:&Entity) -> Rect<f32> {
        match entity.shape {
            Shape::None => Rect { x:0.0, y:0.0, w:0.0, h:0.0 },
            Shape::Sprite { anim_id, .. } | Shape::AnimTiles { anim_id, .. } => {
                let anim = &self.anims[anim_id.get()];
                let frame = anim.frame(self.time);
                Rect {
                    x:entity.pos.x + entity.render_offset.x as f32,
                    y:entity.pos.y + entity.render_offset.y as f32,
                    w:(frame.cols as usize * TILE_WIDTH) as f32,
                    h:(frame.rows as usize * TILE_HEIGHT) as f32,
                }
            },
            Shape::TilemapLayer { tilemap_id } => {
                let tilemap = &self.tilemaps[tilemap_id.get()];
                Rect {
                    x:entity.pos.x + entity.render_offset.x as f32,
                    y:entity.pos.y + entity.render_offset.y as f32,
                    w:tilemap.width(TILE_WIDTH) as f32,
                    h:tilemap.height(TILE_HEIGHT) as f32
                }
            },
        }
    }



    pub fn tile_at(&self, x:f32, y:f32, id:EntityID) -> Option<Tile> {
        let entity = &self.entities[id.get()];
        let Shape::TilemapLayer{ tilemap_id } = entity.shape else { return None };
        
        let tilemap = &self.tilemaps[tilemap_id.get()];
        let rect = self.get_entity_rect(entity);

        if !rect.contains(x, y) { return None };

        let col = ((x - rect.x) as usize / TILE_WIDTH) as u16;
        let row = ((y - rect.y) as usize / TILE_HEIGHT) as u16;

        Some(tilemap.get_tile(col, row))
    }


    // Fills the pixel buffer with current entities
    pub fn render_frame(&mut self){
        // Iterate entities
        for entity in &self.entities {
            if let Shape::None = entity.shape { continue }
            let pos = entity.pos;
            let cam_rect = Rect {
                x: self.cam.x + self.renderer.viewport.x as f32,
                y: self.cam.y + self.renderer.viewport.y as f32,
                w: self.renderer.viewport.w as f32,
                h: self.renderer.viewport.h as f32,
            };

            // Draw entity shape
            match entity.shape {
                Shape::None => {
                    // Do nothing!
                },

                Shape::AnimTiles { anim_id, tilemap_entity, flip_h, flip_v } => {
                    let tilemap_entity = self.get_entity(tilemap_entity);
                    let Shape::TilemapLayer { tilemap_id } = tilemap_entity.shape else { continue };
                    
                    let world_rect = self.get_entity_rect(entity);
                    let Some(vis_rect) = world_rect.intersect(cam_rect) else { continue };
                    
                    let anim = &self.anims[anim_id.get()];
                    let frame = anim.frame(self.time);
                    
                    let tilemap_rect = self.get_entity_rect(tilemap_entity);
                    
                    let tilemap = &mut self.tilemaps[tilemap_id.get()];
                    
                    let left_col = (vis_rect.x - tilemap_rect.x) as i32 / TILE_WIDTH as i32;
                    let top_row = (vis_rect.y - tilemap_rect.y) as i32 / TILE_HEIGHT as i32;
                    
                    tilemap.reset_bg_buffers();
                    tilemap.insert_bg_buffer(left_col as u16, top_row as u16, frame.cols, frame.rows);

                    for row in 0 .. frame.rows as i32 {
                        for col in 0 .. frame.cols as i32 {
                            let mut tile = frame.get_tile(row as u8 * frame.cols + col as u8);
                            //TODO: flipping needs testing
                            tile.set_flipped_h(tile.flipped_h() ^ flip_h);
                            tile.set_flipped_v(tile.flipped_v() ^ flip_v);
                            
                            let tilemap_index = (((row + top_row) * tilemap.cols as i32) + (col + left_col)) as usize;
                            tilemap.tiles[tilemap_index] = tile;
                            // tilemap.tiles[tilemap_index].index = 0;
                        }
                    }
                },

                Shape::Sprite { anim_id, flip_h, .. } => {
                    if !self.draw_sprites { continue }
                    // Draw tiles
                    let anim = &self.anims[anim_id.get()];
                    let frame = anim.frame(self.time);

                    let mut draw_tile = |col:u8, row:u8| {
                        let flipped_col = if flip_h { frame.cols - 1 - col } else { col };
                        let subtile = (row*frame.cols) + flipped_col;
                        let tile = frame.get_tile(subtile);
                        let abs_tile_id = self.renderer.atlas.get_tile_from_tileset(
                            tile.index,
                            anim.tileset
                        );

                        let tile_rect = self.renderer.atlas.rects[abs_tile_id.get()];
                        let quad_rect = Rect{
                            x: pos.x + (col * 8) as f32 + entity.render_offset.x as f32,
                            y: pos.y + (row * 8) as f32 + entity.render_offset.y as f32,
                            w: tile_rect.w as f32,
                            h: tile_rect.h as f32
                        };

                        if !cam_rect.overlaps(&quad_rect) { return }
                        let screen_rect = quad_rect - cam_rect.pos();
                        self.renderer.draw_tile(screen_rect.to_i32(), abs_tile_id, flip_h ^ tile.flipped_h()); //resulting flip is a XOR
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
                    let tilemap = &mut self.tilemaps[tilemap_id.get()];
                    let Some(vis_rect) = world_rect.intersect(cam_rect) else { continue };  

                    // At least a part of tilemap is visible. Render visible tiles within it.
                    let left_col = ((vis_rect.x - world_rect.x) / TILE_WIDTH as f32) as usize;
                    let mut right_col = ((vis_rect.right() - world_rect.x) / TILE_WIDTH as f32) as usize + 1; // +1 prevents cutting off tile too early

                    let top_col = ((vis_rect.y - world_rect.y) / TILE_HEIGHT as f32) as usize;
                    let mut bottom_col = ((vis_rect.bottom() - world_rect.y) / TILE_WIDTH as f32) as usize + 1; // +1 prevents cutting off tile too early

                    // However, those +1's up there will cause invalid coordinates when we reach the end of the tilemap, so...
                    if right_col > tilemap.cols as usize { right_col -= 1 };
                    if bottom_col > tilemap.rows as usize { bottom_col -= 1 };
                    
                    // Acquire and render tiles
                    for row in top_col .. bottom_col {
                        for col in left_col .. right_col {
                            let tile = tilemap.get_tile(col as u16, row as u16);
                            let tile_id = self.renderer.atlas.get_tile_from_tileset(tile.index, tilemap.tileset);

                            let tile_rect = Rect::<i32>::from(self.renderer.atlas.rects[tile.index as usize]);
                            let world_tile_rect = Rect{
                                x: pos.x + (col * TILE_WIDTH) as f32 + entity.render_offset.x as f32 - cam_rect.x,
                                y: pos.y + (row * TILE_HEIGHT) as f32 + entity.render_offset.y as f32 - cam_rect.y,
                                w: tile_rect.w as f32,
                                h: tile_rect.h as f32
                            };
                            self.renderer.draw_tile(world_tile_rect.to_i32(), tile_id, tile.flipped_h());
                        }
                    }
                }
            }

            // Draw collider Wireframe
            #[cfg(debug_assertions)]
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
            #[cfg(debug_assertions)]
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
        #[cfg(debug_assertions)]
        if self.debug_atlas {
            'draw_loop:
            for y in 0 .. ATLAS_HEIGHT {
                for x in 0.. ATLAS_WIDTH {
                    let pixel_index = (y*RENDER_WIDTH) + x;
                    if pixel_index > RENDER_LENGTH - 1 { break 'draw_loop }
                    let atlas_index = (y*ATLAS_WIDTH) + x;
                    self.renderer.pixels[pixel_index] = self.renderer.atlas.pixels[atlas_index]
                }
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


    pub fn start_frame(&mut self) {
        self.elapsed_time_buffer.push(self.time_start_frame.elapsed().as_secs_f32());
        self.time_start_frame = Instant::now();
        
        self.time_elapsed = quantize(self.elapsed_time_buffer.average(), 1.0/ 360.0);
        self.time = self.time_start.elapsed().as_secs_f32();

        // for tilemap in self.tilemaps.iter_mut() {
        //     tilemap.reset_bg_buffers()
        // }

    }



    pub fn finish_frame(&mut self) {
        self.time_update = self.time_start_frame.elapsed().as_secs_f32();
        self.time_update_buffer.push(self.time_update);

        // Limit frame rate. TODO: This is hacky, doesn't always work.
        if let Some(fps_limit) = self.limit_frame_rate {
            let immediate_fps = 1.0 / self.time_update;
            if immediate_fps > fps_limit {
                let time_target = 1.0 / fps_limit;
                let time_diff = time_target - self.time_update;
                if time_diff > 1.0 / 240.0 {
                    self.time_idle = time_diff * 0.75;
                    sleep(Duration::from_secs_f32(self.time_idle));
                }
            } else {
                println!("Skipping idle cycle!");
                self.time_idle = 0.0;
            }
        } else {
            self.time_idle = 0.0;
        }
    }

}




