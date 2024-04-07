use crate::*;

const COL_MARGIN:f32 = 0.02;

impl<T,P> World<T,P>
where T:TilesetEnum, P:PaletteEnum {

    /// Performs the transform with the appropriate collision reaction, taking into account the velocity and the current frame's delta-time.
    /// Returns the collision data if any occurred.
    pub fn move_with_collision( &mut self, entity_id: EntityID, velocity:Vec2<f32>, reaction:CollisionReaction) -> Option<Collision<f32>> {
        if !self.entities.contains_key(entity_id) { return None }
        
        // Passed to all collision calculations witn frame delta already applied
        let scaled_velocity = velocity.scale(self.time_elapsed);

        // Run collision code if probe is found
        let collider = self.colliders.get(entity_id)?;
        let pos = self.entities[entity_id].pos;
        if let Some(mut probe) = Self::probe_get_from_collider(collider, pos, entity_id, Some(scaled_velocity)) {

            // Modified on every collision
            let mut col_accumulator:Option<Collision<f32>> = None;

            for layer in 0 .. self.collision_layers.len() as u32 {
                let bitmask = 1 << layer;
                if probe.mask as u32 & bitmask != 0 {
                    for other_probe in self.collision_layers[layer as usize].values(){     
                        if other_probe.entity_id == entity_id { continue }

                        let maybe_col = match other_probe.kind {
                            ColliderKind::Point | ColliderKind::Rect { .. } => {
                                probe.collision_response(other_probe, None)
                            },
                            ColliderKind::Tilemap { .. } => {
                                let Shape::Bg { tileset_id, tilemap_id } = &self.entities[other_probe.entity_id].shape else { continue };
                                let tilemap = self.renderer.get_tilemap(*tileset_id, *tilemap_id);
                                probe.collision_response(other_probe, Some(tilemap))
                            },
                        };
        
                        // Accumulate collisions, if more than one occurred
                        if let Some(mut current_col) = maybe_col {
                            if let ColliderKind::Tilemap{ tile_width, tile_height, .. } = other_probe.kind {
                                // Scale tile to world coordinates
                                current_col.pos.x *= tile_width as f32;
                                current_col.pos.y *= tile_height as f32;
                            }
                            if let Some(ref mut col) = col_accumulator {
                                *col += current_col;
                            } else {
                                col_accumulator = Some(current_col);
                            }
                        }
                    }
                }
            }

            // Process accumulated collisions, if any
            if let Some(collision) = &col_accumulator {
                // println!("{:.1?}", collision);
                // Reaction
                let (delta, vel) = {
                    match reaction {
                        CollisionReaction::None => {(
                            scaled_velocity,
                            scaled_velocity
                        )},
                        CollisionReaction::Bounce(amount) => {(
                            Vec2{
                                x: collision.t.x.clamp(-1.0, 1.0) * scaled_velocity.x,
                                y: collision.t.y.clamp(-1.0, 1.0) * scaled_velocity.y
                            },
                            Vec2::reflect(probe.velocity.scale(amount), collision.normal)
                        )},
                        CollisionReaction::Slide => {
                            (
                                Vec2{
                                    x: collision.t.x.clamp(-1.0, 1.0) * scaled_velocity.x,
                                    y: collision.t.y.clamp(-1.0, 1.0) * scaled_velocity.y
                                },
                                collision.velocity
                            )
                        }
                    }
                };

                // Stick to other surface if moving (i.e. on top of platform)
                let stick_delta = match reaction {
                    CollisionReaction::None => Vec2::zero(),
                    CollisionReaction::Bounce(_) |  CollisionReaction::Slide => collision.velocity,
                };

                // Inherit collision speed?
                let add_vel = match reaction {
                    CollisionReaction::None => Vec2::zero(),
                    CollisionReaction::Bounce(_) | CollisionReaction::Slide => collision.velocity
                };

                // Apply new position
                let pos = &mut self.entities[entity_id].pos;
                let margin = Vec2{
                    x: COL_MARGIN * collision.normal.x,
                    y: COL_MARGIN * collision.normal.y
                };
                *pos += delta + stick_delta + margin;

                // Return collision
                let unscale = 1.0 / self.time_elapsed;

                // Update probe position for proper debug display (WIP)
                let collider = self.colliders.get(entity_id).unwrap();
                probe.pos = *pos + collider.offset;
                self.probe_add(probe);

                // TODO: return more than one collision? Only the first one is reported,
                // although "t", "pos" and "normal" take into account multiple collisions
                return Some(Collision{
                    t: collision.t,
                    pos: collision.pos,                           
                    tile: collision.tile,
                    normal: collision.normal,
                    velocity: (vel + add_vel).scale(unscale),     // returns "unscaled" velocity, since the caller passed it like that
                    colliding_entity: collision.colliding_entity,             
                })
            } 

            probe.pos += scaled_velocity;
            self.probe_add(probe);
        }
    
        // Update current position
        self.entities[entity_id].pos += scaled_velocity;
        // Update probe with current position

        None
    }

    
    /// Removes the collider
    pub fn collider_remove(&mut self, id:EntityID) {
        // Remove from probes to avoid collision on this frame
        if let Some(collider) = self.colliders.get(id){
            if collider.layer > 0 {
                let layer = collider.layer as usize;
                self.collision_layers[layer-1].remove(id);
            }
        }
        // Then remove from collider map
        self.colliders.remove(id);
    }


    /// Adds a collider to the entity. Active colliders are intended to be moved using "move_with_collision", while
    /// passive colliders are mostly used for static elements, but can be used if the entity is being moved manually
    /// and is only collided against.
    pub fn collider_add(&mut self, id:EntityID, collider:Collider<f32>) {
        self.colliders.insert(id, collider);
        self.collider_update(id, None);
    }


    /// Call if you have manually updated an entity position without using "move_with_collision"
    /// If you set the collider to passive, it will be automatically updated and this is not necessary,
    /// But it may introduce a one frame delay. Calling collider_update ensures an active collider is updated
    /// on the same frame it is moved.
    pub fn collider_update(&mut self, id:EntityID, velocity:Option<Vec2<f32>>) -> Option<()> {
        let collider = self.colliders.get(id)?;
        let pos = self.get_position(id);
        if let Some(mut probe) = Self::probe_get_from_collider(collider, pos, id, velocity){
            if probe.layer > 0 {
                let layer = probe.layer as usize;
                self.probe_process(&mut probe);
                self.collision_layers[layer-1].insert(probe.entity_id, probe);
            }
        }
        Some(())
    }


    /// Modifies the collision mask of an entity.
    pub fn enable_collision_with_layer(&mut self, id:EntityID, layer:impl CollisionLayer) {
        if let Some(collider) = self.colliders.get_mut(id){
            let layer_value:u16 = layer.into();
            if layer_value > 0 {
                let layer = 2u16.pow(layer_value as u32 - 1);
                collider.mask |= layer;
                self.collider_update(id, None);
                // println!("Enabling collision with layer {:08b}", layer_value);
            } else {
                #[cfg(feature = "std")]{
                    println!("World: Warning, can't enable collision to layer 0 ({:?})! Suggestions:", layer);
                    println!("       - Set collider.enabled to 'false' instead.");
                    println!("       - If collision to {:?} is what you want, make sure your collision layers contain a 'None' variant!", layer);
                }
            }
        } else {
            #[cfg(feature = "std")]{
                println!("World: Warning, collider {:?} not found", id);
            }
        }
    }


    /// If the entity has a Bg Shape, returns the tile at the specified world coordinates, if any.
    pub fn tile_at(&self, x: f32, y: f32, id: EntityID) -> Option<(Tile, Rect<f32>)> {
        let (tilemap, tilemap_rect) = self.get_tilemap_and_rect(id)?;
        if !tilemap_rect.contains(x, y) { return None };

        let col = (u16::try_from((x - tilemap_rect.x) as usize / self.specs.tile_width as usize))
            .unwrap();
        let row = (u16::try_from((y - tilemap_rect.y) as usize / self.specs.tile_height as usize))
            .unwrap();

        let w = self.specs.tile_width as f32;
        let h = self.specs.tile_height as f32;
        let tile_rect = Rect {
            x: tilemap_rect.x + (col as f32 * w),
            y: tilemap_rect.y + (row as f32 * h),
            w,
            h,
        };

        Some((tilemap.get_tile(col, row), tile_rect))
    }


    pub fn get_collision_layers(&self) -> &ProbeMap {
        &self.collision_layers
    }


    pub fn get_colliders(&self) -> &ColliderMap {
        &self.colliders
    }



    pub(crate) fn probe_process(&self, probe:&mut CollisionProbe<f32>) {
        if let ColliderKind::Tilemap {ref mut w, ref mut h, ref mut tile_width, ref mut tile_height } = probe.kind {
            let rect = self.get_entity_rect_from_id(probe.entity_id);
            *w = rect.w;
            *h = rect.h;
            *tile_width = self.specs.tile_width;
            *tile_height = self.specs.tile_height;
        }
    }


    pub(crate) fn probe_add(&mut self, mut probe:CollisionProbe<f32>) {
        if probe.layer > 0 {
            let layer = probe.layer as usize;
            self.probe_process(&mut probe);
            // self.collision_layers[layer-1].push(probe);
            self.collision_layers[layer-1].insert(probe.entity_id, probe);
        }
    }


    pub(crate) fn probe_get_from_collider(collider:&Collider<f32>, pos: Vec2<f32>, id:EntityID, velocity:Option<Vec2<f32>>) -> Option<CollisionProbe<f32>>  {
        if !collider.enabled { return None }
        Some(CollisionProbe{
            entity_id: id,
            pos: pos + collider.offset,
            velocity: velocity.unwrap_or_default(),
            kind: collider.kind,
            layer: collider.layer,
            mask: collider.mask,
        })
    }

    
}