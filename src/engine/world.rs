use crate::*;
use alloc::{vec, vec::Vec};
use slotmap::{SecondaryMap, SlotMap};

const COLLISION_LAYER_COUNT:usize = 16;

pub(crate) type ColliderMap = SecondaryMap<EntityID, Collider<f32>>;
pub(crate) type ProbeMap = Vec<SecondaryMap<EntityID, CollisionProbe<f32>>>;

mod collisions;
mod frame_events;
mod render;
mod transform;

/// A World contains all necessary data to render and detect collisions on entities, including the
/// tile Renderer and associated data like Tilemaps and Animations.
pub struct World<T, P>
where T:TilesetEnum, P:PaletteEnum,
{
    // Visible to Host App
    pub limit_frame_rate: Option<f32>,
    pub debug_colliders: bool,
    pub debug_wireframe: bool,
    pub debug_atlas: bool,
    pub draw_sprites: bool,
    pub draw_tilemaps: bool,
    pub cam: Rect<f32>,

    // Main components
    pub framebuf: FrameBuf,
    pub renderer: Renderer<T, P>,

    // Private
    specs: Specs,
    time_elapsed_buffer: SmoothBuffer<15>, // Affects gameplay speed (used to calculate frame deltas)
    time_update_buffer: SmoothBuffer<120>, // For performance info only, doesn't affect gameplay
    time: f32,
    time_update: f32,
    time_elapsed: f32,
    time_idle: f32,

    // collision masks only allow 8 layers for now (8 bit mask, each bit determines a layer collision)
    collision_layers:ProbeMap,
    colliders: ColliderMap,

    // Data Pools
    entities:SlotMap<EntityID, Entity>,
}

impl<T, P> World<T, P>
where T:TilesetEnum, P:PaletteEnum,
{
    pub fn new(specs:Specs) -> Self {
        World {
            limit_frame_rate: None,
            debug_colliders: false,
            debug_wireframe: false,
            debug_atlas: false,
            draw_sprites: true,
            draw_tilemaps: true,


            cam: Rect::new(0.0, 0.0, specs.render_width as f32, specs.render_height as f32),
            framebuf: FrameBuf::new(specs),
            renderer: Renderer::new(specs),

            time_elapsed_buffer: SmoothBuffer::new(),
            time_update_buffer: SmoothBuffer::new(),
            time: 0.0,
            time_update: 1.0 / 60.0,

            time_elapsed: 1.0 / 60.0,
            time_idle: 0.0,

            entities: Default::default(),

            colliders: Default::default(),
            collision_layers: vec![SecondaryMap::default(); COLLISION_LAYER_COUNT],
            specs,
        }
    }


    // *************************************** Timing **************************************


    pub fn time(&self) -> f32 {
        self.time
    }


    pub fn fps(&self) -> f32 {
        1.0 / self.time_elapsed
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


    // ************************************ Scene Management ***********************************


    pub fn reset(&mut self) {
        self.renderer.reset();
        self.entities.clear();
        self.colliders.clear();
        self.collision_layers.iter_mut().for_each(|layer|{
            layer.clear();
        })
    }


    pub fn entity_remove(&mut self, id: EntityID) {
        if let Some(ent) = self.entities.get(id) {
            // Clean up BgTiles if needed.
            // Tilemap will be left "dirty" by the AnimTile entity if this is not performed
            if let Shape::BgSprite { tilemap_entity, .. } = ent.shape {
                if let Some(tilemap_ent) = self.entities.get(tilemap_entity) {
                    if let Shape::Bg {
                        tileset_id,
                        tilemap_id,
                    } = tilemap_ent.shape
                    {
                        let tilemap = self.renderer.get_tilemap_mut(tileset_id, tilemap_id);
                        tilemap.restore_bg_buffer(ent.id);
                        tilemap.bg_buffers.remove(id);
                    }
                }
            }
            self.entities.remove(id);
            self.collider_remove(id);
        }
    }


    pub fn entity_add(&mut self, depth:u8) -> EntityID {
        self.entities.insert_with_key(|key|{
            Entity::new(key, depth)
        })
    }


    pub fn entities(&self) -> &SlotMap<EntityID, Entity> {
        &self.entities
    }


    pub fn get_entity(&self, id: EntityID) -> Option<&Entity> {
        self.entities.get(id)
    }


    pub fn get_entity_mut(&mut self, id: EntityID) -> Option<&mut Entity> {
        self.entities.get_mut(id)
    }


    /// Allows "breaking" the mutable refs per field, makes it a little easier to please the borrow checker in some cases
    pub fn get_members(&mut self) -> (
        &mut SlotMap<EntityID, Entity>,
        &mut Renderer<T,P>)
    {(
        &mut self.entities,
        &mut self.renderer
    )}


    // fn get_collision_members(&self) -> (
    //     &SlotMap<EntityID, Entity>,
    //     &ColliderMap,
    //     &Vec<Vec<CollisionProbe<f32>>>,
    // )
    // {(
    //     &self.entities,
    //     &self.colliders,
    //     &self.collision_layers,
    // )}

}
