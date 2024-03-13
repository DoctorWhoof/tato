use tato::*;
use macroquad::input::*;
use crate::{Game, TilesetID};


pub fn frame(game:&mut Game) {
    // Debug Input
    if is_key_pressed(KeyCode::A) { game.world.debug_atlas = !game.world.debug_atlas }
    if is_key_pressed(KeyCode::W) { game.world.debug_pivot = !game.world.debug_pivot }

    // Player
    let speed = 120.0;
    let bullet_speed = 240.0;
    let bounds = Rect{ x: 8.0, y:8.0, w: 240.0, h: 176.0 };

    if is_key_down(KeyCode::Right) {
        game.player.vel.x = speed
    } else if is_key_down(KeyCode::Left) {
        game.player.vel.x = -speed
    } else{
        game.player.vel.x = 0.0
    }

    if is_key_down(KeyCode::Up) {
        game.player.vel.y = -speed
    } else if is_key_down(KeyCode::Down) {
        game.player.vel.y = speed
    } else {
        game.player.vel.y = 0.0
    }

    game.world.move_with_collision(game.player.id, game.player.vel, CollisionReaction::None);
    if let Some(ent) = game.world.get_entity_mut(game.player.id){
        ent.pos = ent.pos.clamp_to_rect(bounds);
    }

    // Fire Shots
    if is_key_down(KeyCode::Z) && game.cooldown <= 0.0 {
        game.cooldown = 0.1;
        let pos = game.world.get_position(game.player.id);

        let bullet = game.world.add_entity(0);
        game.world.add_collider(bullet, Collider::new_rect_collider(-2.0, -2.0, 4.0, 4.0));
        game.world.set_render_offset(bullet, -3, -2);
        game.world.set_position(bullet, pos);
        game.world.set_shape(bullet, Shape::Sprite {
            tileset:TilesetID::Player.into(),
            anim_id:1,
            flip_h: false,
            flip_v: false 
        });

        // Pushing a new bullet may remove an older one if capacity is reached
        if let Some(removed_bullet) = game.bullets.push(bullet) {
            game.world.remove_entity(removed_bullet);
        }
    } else {
        // Move timer closer to zero (zero or less means Ok to shoot)
        game.cooldown -=1.0 * game.world.time_elapsed()
    }

    // Iterate bullets, remove any too far in the y coordinate.
    game.bullets.retain(|id|{
        if game.world.get_position(*id).y > -8.0 {
            // Move bullet
            game.world.translate(*id, Vec2 { x: 0.0, y: -bullet_speed });
            true
        } else {
            // Destroy bullet
            game.world.remove_entity(*id);
            false
        }
    });

    // BG
    let bg_speed = 15.0;
    let height = game.world.framebuf.height() as f32;
    let elapsed = game.world.time_elapsed();

    let mut scroll = | id:EntityID, speed: f32 | {
        if let Some(ent) = game.world.get_entity_mut(id){
            if ent.pos.y > height { ent.pos.y = -height }
            ent.pos.y += speed * elapsed
        }
    };

    scroll(game.stars_bg_0, bg_speed);
    scroll(game.stars_bg_1, bg_speed);
    scroll(game.stars_fg_0, bg_speed * 2.0);
    scroll(game.stars_fg_1, bg_speed * 2.0);

    if let Some(ent) = game.world.get_entity_mut(game.stars_bg_0){
        if ent.pos.y > height { ent.pos.y = -height }
        ent.pos.y += bg_speed * elapsed
    }

    if let Some(ent) = game.world.get_entity_mut(game.stars_bg_1){
        if ent.pos.y > height { ent.pos.y = -height }
        ent.pos.y += bg_speed * elapsed
    }

    if let Some(ent) = game.world.get_entity_mut(game.stars_fg_0){
        if ent.pos.y > height { ent.pos.y = -height }
        ent.pos.y += bg_speed * 2.0 * elapsed
    }

    if let Some(ent) = game.world.get_entity_mut(game.stars_fg_1){
        if ent.pos.y > height { ent.pos.y = -height }
        ent.pos.y += bg_speed * 2.0 * elapsed
    }

}

