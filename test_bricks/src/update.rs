use crate::{GameWorld, Paddle, Puck};
use spud::{CollisionReaction, Vec2};

pub fn move_player(paddle: &mut Paddle, world: &mut GameWorld) {
    let speed_x = 20.0;
    let speed_y = 10.0;

    let max_speed_x = 120.0;
    let max_speed_y = 120.0;

    let min_x = 20.0;
    let max_x = 236.0;
    let min_y = 8.0;
    let max_y = 186.0;

    if paddle.input.left {
        paddle.vel.x -= speed_x;
    } else if paddle.input.right {
        paddle.vel.x += speed_x;
    } else if paddle.vel.x > 0.0 {
        paddle.vel.x -= speed_x;
    } else if paddle.vel.x < 0.0 {
        paddle.vel.x += speed_x;
    }

    if paddle.input.up {
        paddle.vel.y -= speed_y;
    } else if paddle.input.down {
        paddle.vel.y += speed_y;
    } else if paddle.vel.y > 0.0 {
        paddle.vel.y -= speed_y;
    } else if paddle.vel.y < 0.0 {
        paddle.vel.y += speed_y;
    }

    paddle.vel.x = paddle.vel.x.clamp(-max_speed_x, max_speed_x);
    paddle.vel.y = paddle.vel.y.clamp(-max_speed_y, max_speed_y);

    world.move_with_collision(paddle.id, paddle.vel, CollisionReaction::Slide);

    if let Some(ent) = world.get_entity_mut(paddle.id) {
        if ent.pos.x > max_x {
            paddle.vel.x = 0.0;
            ent.pos.x = max_x;
        } else if ent.pos.x < min_x {
            paddle.vel.x = 0.0;
            ent.pos.x = min_x;
        }

        if ent.pos.y > max_y {
            paddle.vel.y = 0.0;
            ent.pos.y = max_y;
        } else if ent.pos.y < min_y {
            paddle.vel.y = 0.0;
            ent.pos.y = min_y;
        }
    }
}


pub fn move_puck(puck: &mut Puck, world: &mut GameWorld) {
    let max_speed = 120.0;
    let safety_speed = 180.0;
    let deccelerate_rate = 15.0;
    let elapsed = world.time_elapsed();

    puck.vel = puck.vel.clamp_to_length(safety_speed);
    if puck.vel.len() > max_speed {
        // println!("slow down!: {:?}", puck.vel.len());
        if puck.vel.x > 0.0 {
            puck.vel.x -= deccelerate_rate * elapsed;
        } else {
            puck.vel.x += deccelerate_rate * elapsed;
        }
        if puck.vel.y > 0.0 {
            puck.vel.y -= deccelerate_rate * elapsed;
        } else {
            puck.vel.y += deccelerate_rate * elapsed;
        }
    }

    if let Some(col) = world.move_with_collision(puck.id, puck.vel, CollisionReaction::Bounce(1.0)){
        puck.vel = col.velocity
    };

    // Out of bounds reset
    if let Some(ent) = world.get_entity_mut(puck.id) {
        if ent.pos.x < 0.0 || ent.pos.x > 256.0 || ent.pos.y < 0.0 || ent.pos.y > 190.0 {
            puck.vel = Vec2::default();
            ent.pos = puck.initial_pos;
        }
    }
}
