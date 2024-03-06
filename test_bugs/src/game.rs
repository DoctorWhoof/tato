use tato::*;
use macroquad::input::*;
use crate::{GameWorld, GameAtlas, TilesetID, Player};

pub struct Game {
    pub world:GameWorld,
    pub atlas:GameAtlas,
    player: Player,
    human: EntityID,
    // enemies: Vec<EntityID>, 
    stars_bg_0:EntityID,
    stars_bg_1:EntityID,
    stars_fg_0:EntityID,
    stars_fg_1:EntityID
}

impl Game {

    // **************************************** Init ****************************************

    
    pub fn new() -> Self{
        // Spud init
        let mut world: GameWorld = World::new();
        let atlas = GameAtlas::load( include_bytes!("../assets/converted/atlas") );
        world.render.load_palettes_from_atlas(&atlas);  // TODO: get rid of this step... I always forget it!
        world.render.load_tileset(&atlas, TilesetID::Bg);
        world.render.load_tileset(&atlas, TilesetID::Player);
        world.render.load_tileset(&atlas, TilesetID::Enemies);
    
        let stars_bg_0 = world.add_entity(0);
        world.set_shape(stars_bg_0, Shape::Bg{ tileset: TilesetID::Bg.into(), tilemap_id: 0 });

        let stars_bg_1 = world.add_entity(0);
        world.set_shape(stars_bg_1, Shape::Bg{ tileset: TilesetID::Bg.into(), tilemap_id: 0 });
        world.set_position(stars_bg_1, Vec2{x:0.0, y:-192.0});
    
        let stars_fg_0 = world.add_entity(0);
        world.set_shape(stars_fg_0, Shape::Bg{ tileset: TilesetID::Bg.into(), tilemap_id: 1 });

        let stars_fg_1 = world.add_entity(0);
        world.set_shape(stars_fg_1, Shape::Bg{ tileset: TilesetID::Bg.into(), tilemap_id: 1 });
        world.set_position(stars_fg_1, Vec2{x:0.0, y:-192.0});
    
        let player = Player {
            id: {
                let id = world.add_entity(0);
                world.set_shape(id, Shape::sprite_from_anim(TilesetID::Player, 0));
                world.set_position(id, Vec2::new(128.0, 160.0));
                world.set_render_offset(id, -8, -8);
                id
            },
            health: 10,
            score: 0,
            vel: Vec2::zero(),
        };
    
        let human = world.add_entity(0);
        world.set_shape(human, Shape::sprite_from_anim(TilesetID::Enemies, 0));
        world.set_position(human, Vec2::new(128.0, 32.0));
        world.set_render_offset(human, -8, -8);

        Self{
            world,
            atlas,
            player,
            human,
            stars_bg_0, stars_bg_1, stars_fg_0, stars_fg_1
        }
    }


    // **************************************** Update ****************************************


    pub fn update(&mut self) {
        let speed = 120.0;
        let bounds = Rect{ x: 8.0, y:8.0, w: 240.0, h: 176.0 };

        if is_key_down(KeyCode::Right) {
            self.player.vel.x = speed
        } else if is_key_down(KeyCode::Left) {
            self.player.vel.x = -speed
        } else{
            self.player.vel.x = 0.0
        }

        if is_key_down(KeyCode::Up) {
            self.player.vel.y = -speed
        } else if is_key_down(KeyCode::Down) {
            self.player.vel.y = speed
        } else {
            self.player.vel.y = 0.0
        }

        self.world.move_with_collision(self.player.id, self.player.vel, CollisionReaction::None);
        if let Some(ent) = self.world.get_entity_mut(self.player.id){
            ent.pos = ent.pos.clamp_to_rect(bounds);
        }

        // BG
        let bg_speed = 15.0;
        let height = self.world.framebuf.height() as f32;
        let elapsed = self.world.time_elapsed();

        if let Some(ent) = self.world.get_entity_mut(self.stars_bg_0){
            if ent.pos.y > height { ent.pos.y = -height }
            ent.pos.y += bg_speed * elapsed
        }

        if let Some(ent) = self.world.get_entity_mut(self.stars_bg_1){
            if ent.pos.y > height { ent.pos.y = -height }
            ent.pos.y += bg_speed * elapsed
        }

        if let Some(ent) = self.world.get_entity_mut(self.stars_fg_0){
            if ent.pos.y > height { ent.pos.y = -height }
            ent.pos.y += bg_speed * 4.0 * elapsed
        }

        if let Some(ent) = self.world.get_entity_mut(self.stars_fg_1){
            if ent.pos.y > height { ent.pos.y = -height }
            ent.pos.y += bg_speed * 4.0 * elapsed
        }


    }
}


impl Default for Game { fn default() -> Self { Self::new() } }