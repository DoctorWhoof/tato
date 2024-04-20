use tato::*;
use macroquad::input::*;
use crate::{Game, Layer, TilesetID};


pub fn frame(game:&mut Game) {
    // Debug Input
    if is_key_pressed(KeyCode::A) { game.world.debug_atlas = !game.world.debug_atlas }
    if is_key_pressed(KeyCode::W) { game.world.debug_wireframe = !game.world.debug_wireframe }
    if is_key_pressed(KeyCode::C) { game.world.debug_colliders = !game.world.debug_colliders }

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
        ent.pos = bounds.clamp_vec(ent.pos);
    }

    // *********************************************************************************************************************
    // TODO: Add a world collision step (after "update" step) so that the Order of updating does NOT affect collisions! 
    // Currently if I create the probe after the other colliders, no collision happens!
    // *********************************************************************************************************************

    // Enemies
    let time = game.world.time();
    game.enemies.pos.x = (time.sin() * 16.0) + 128.0;

    let spacing = Vec2::new(32.0, 24.0);
    let center_x = ((game.enemies.size().x as f32 - 1.0) * spacing.x) / 2.0;
    let center_y = ((game.enemies.size().y as f32 - 1.0) * spacing.y) / 2.0;
    
    for col in 0 .. game.enemies.size().x {
        for row in 0 .. game.enemies.size().y {
            if let Some(id) = game.enemies.get(col, row) {
                let Some(ent) = game.world.get_entity_mut(id) else { continue };
                let columm_wave = (time + (col + 1) as f32) * 4.0;
                let y_offset = columm_wave.sin() * 4.0;
                let x = game.enemies.pos.x + (col as f32 * spacing.x) - center_x;
                let y = game.enemies.pos.y + (row as f32 * spacing.y) - center_y + y_offset;
                ent.pos = Vec2::new(x, y);
                // We need to manually update the collider since we're "teleporting" the entities to new coordinates
                game.world.collider_update(id, None);
            }
        }
    }

    // Fire Shots
    if is_key_pressed(KeyCode::Z) {
    // if is_key_down(KeyCode::Z) && game.cooldown <= 0.0 {
    //     game.cooldown = 0.2;
        let pos = game.world.get_position(game.player.id);
        let bullet = game.world.entity_add(1);

        // TODO: new_collider(layer, col:impl ToCollider), in order to remove individual collider creation functions
        let collider = Collider::from_rect(Layer::Bullet, Rect::new(-2.0, 0.0, 4.0, 4.0));
        game.world.collider_add(bullet, collider);
        game.world.enable_collision_with_layer(bullet, Layer::Enemies);

        game.world.set_render_offset(bullet, -3, -2);
        game.world.set_position(bullet, pos);
        game.world.set_shape(bullet, Shape::Sprite {
            tileset_id:TilesetID::Player.into(),
            anim_id:1,
            flip_h: false,
            flip_v: false 
        });

        // Pushing a new bullet may remove an older one if capacity is reached
        if let Some(removed_bullet) = game.bullets.push(bullet) {
            game.world.entity_remove(removed_bullet);
        }
    }
    // } else {
    //     // Move timer closer to zero (zero or less means Ok to shoot)
    //     game.cooldown -=1.0 * game.world.time_elapsed()
    // }


    // Iterate bullets, remove any too far in the y coordinate.
    game.bullets.retain(|id|{
        if game.world.get_position(*id).y > -8.0 {
            // Move bullet
            if let Some(col) = game.world.move_with_collision(*id, Vec2 { x: 0.0, y: -bullet_speed }, CollisionReaction::None){
                // Target hit, Destroy bullet
                game.world.entity_remove(*id);
                game.world.entity_remove(col.colliding_entity);
                // TODO: Remove entry from enemies, based on coordinates?
                false
            } else {
                true
            }
        } else {
            // Destroy bullet
            game.world.entity_remove(*id);
            false
        }
    });

    // BG
    let bg_speed = 15.0;
    let elapsed = game.world.time_elapsed();

    let mut scroll = | id:EntityID, speed: f32 | {
        let height = game.world.get_entity_rect_from_id(id).h;
        if let Some(ent) = game.world.get_entity_mut(id){
            ent.pos.y += speed * elapsed;
            if ent.pos.y > height { ent.pos.y = -height }
        }
    };

    scroll(game.stars_bg_0, bg_speed);
    scroll(game.stars_bg_1, bg_speed);
    scroll(game.stars_fg_0, bg_speed * 2.0);
    scroll(game.stars_fg_1, bg_speed * 2.0);

    // Test
    // game.world.center_camera_on(game.player.id);

}

