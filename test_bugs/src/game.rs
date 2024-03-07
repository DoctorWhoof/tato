use tato::*;
use macroquad::input::*;
use crate::{GameWorld, GameAtlas, TilesetID, Player};

pub struct Game {
    pub world:GameWorld,
    pub atlas:GameAtlas,
    player: Player,
    // human: EntityID,
    // enemies: Vec<EntityID>, 
    stars_bg_0:EntityID,
    stars_bg_1:EntityID,
    stars_fg_0:EntityID,
    stars_fg_1:EntityID,
    pub cooldown:f32,
    pub bullets:RingPool<EntityID, 16>,
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

        let bullets = RingPool::new();

        Self{
            world,
            atlas,
            player,
            bullets,
            stars_bg_0, stars_bg_1, stars_fg_0, stars_fg_1,
            cooldown: 0.0
        }
    }


    // **************************************** Update ****************************************


    pub fn update(&mut self) {
        let speed = 120.0;
        let bullet_speed = 240.0;
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

        // Fire Shots
        if is_key_down(KeyCode::Z) && self.cooldown <= 0.0 {
            self.cooldown = 0.1;
            let pos = self.world.get_position(self.player.id);

            let bullet = self.world.add_entity(0);
            self.world.add_collider(bullet, Collider::new_rect_collider(-2.0, -2.0, 4.0, 4.0));
            self.world.set_render_offset(bullet, -3, -2);
            self.world.set_position(bullet, pos);
            self.world.set_shape(bullet, Shape::Sprite {
                tileset:TilesetID::Player.into(),
                anim_id:1,
                flip_h: false,
                flip_v: false 
            });

            // Pushing a new bullet may remove an older one if capacity is reached
            if let Some(removed_bullet) = self.bullets.push(bullet) {
                self.world.remove_entity(removed_bullet);
            }
        } else {
            // Move timer closer to zero (zero or less means Ok to shoot)
            self.cooldown -=1.0 * self.world.time_elapsed()
        }

        // Iterate bullets, remove any too far in the y coordinate.
        self.bullets.retain(|id|{
            if self.world.get_position(*id).y > -8.0 {
                // Move bullet
                self.world.translate(*id, Vec2 { x: 0.0, y: -bullet_speed });
                true
            } else {
                // Destroy bullet
                self.world.remove_entity(*id);
                false
            }
        });

        // BG
        let bg_speed = 15.0;
        let height = self.world.framebuf.height() as f32;
        let elapsed = self.world.time_elapsed();

        let mut scroll = | id:EntityID, speed: f32 | {
            if let Some(ent) = self.world.get_entity_mut(id){
                if ent.pos.y > height { ent.pos.y = -height }
                ent.pos.y += speed * elapsed
            }
        };

        scroll(self.stars_bg_0, bg_speed);
        scroll(self.stars_bg_1, bg_speed);
        scroll(self.stars_fg_0, bg_speed * 2.0);
        scroll(self.stars_fg_1, bg_speed * 2.0);

        // if let Some(ent) = self.world.get_entity_mut(self.stars_bg_0){
        //     if ent.pos.y > height { ent.pos.y = -height }
        //     ent.pos.y += bg_speed * elapsed
        // }

        // if let Some(ent) = self.world.get_entity_mut(self.stars_bg_1){
        //     if ent.pos.y > height { ent.pos.y = -height }
        //     ent.pos.y += bg_speed * elapsed
        // }

        // if let Some(ent) = self.world.get_entity_mut(self.stars_fg_0){
        //     if ent.pos.y > height { ent.pos.y = -height }
        //     ent.pos.y += bg_speed * 2.0 * elapsed
        // }

        // if let Some(ent) = self.world.get_entity_mut(self.stars_fg_1){
        //     if ent.pos.y > height { ent.pos.y = -height }
        //     ent.pos.y += bg_speed * 2.0 * elapsed
        // }

    }
}


impl Default for Game { fn default() -> Self { Self::new() } }