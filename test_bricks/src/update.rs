use crate::{GameWorld, Paddle, Puck};
use spud::{EntityID, Vec2};
// use spud::{mirror, Collision, EntityID, Tile, Vec2, RAD_TO_DEG};

pub fn move_player(paddle: &mut Paddle, world: &mut GameWorld) {
    let elapsed = world.time_elapsed();

    let Some(ent) = world.get_entity_mut(paddle.id) else {
        return;
    };
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

    ent.pos.x += paddle.vel.x * elapsed;
    ent.pos.y += paddle.vel.y * elapsed;

    // Limits
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

    // Remove any residual velocity
    if paddle.vel.x < speed_x && paddle.vel.x > -speed_x {
        paddle.vel.x = 0.0;
    }

    if paddle.vel.y < speed_y && paddle.vel.y > -speed_y {
        paddle.vel.y = 0.0;
    }
}


pub fn move_puck(puck: &mut Puck, paddle: &Paddle, world: &mut GameWorld) {
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

    world.move_and_collide(puck.id, &mut puck.vel, paddle.id, paddle.vel);

    if let Some(ent) = world.get_entity_mut(puck.id) {
        if ent.pos.x < 0.0 || ent.pos.x > 256.0 || ent.pos.y < 0.0 || ent.pos.y > 190.0 {
            puck.vel = Vec2::default();
            ent.pos = puck.initial_pos;
        }
    }
}

// TODO: Move to engine math

// fn point_to_rect_normal(x:f32, y:f32, rect: &Rect<f32>) -> f32 {
//     let distance_top = (y - rect.y).abs().floor();
//     let distance_bottom = (y - rect.bottom()).abs().floor();
//     let distance_left = (x - rect.x).abs();
//     let distance_right = (x - rect.right()).abs();

//     let min_distance = f32::min(
//         f32::min(distance_top, distance_bottom),
//         f32::min(distance_left, distance_right),
//     );

//     const RIGHT:f32 = 0.0 * DEG_TO_RAD;
//     const UP:f32 = 90.0 * DEG_TO_RAD;
//     const LEFT:f32 = 180.0 * DEG_TO_RAD;
//     const DOWN:f32 = 270.0 * DEG_TO_RAD;

//     if min_distance == distance_top {
//         println!("Top");
//         UP
//     } else if min_distance == distance_bottom {
//         println!("Bottom");
//         DOWN
//     } else if min_distance == distance_left {
//         println!("Left");
//         LEFT
//     } else {
//         println!("Right");
//         RIGHT
//     }
// }

// fn average_angle(a: f32, b: f32) -> f32 {
//     println!("average between {a} and {b}");
//     // Convert angles to 2D unit vectors on the unit circle
//     let vec1 = (a.cos(), a.sin());
//     let vec2 = (b.cos(), b.sin());

//     // Calculate the average vector
//     let avg_vec = ((vec1.0 + vec2.0) / 2.0, (vec1.1 + vec2.1) / 2.0);

//     // Calculate the average angle from the average vector
//     let avg_angle = avg_vec.1.atan2(avg_vec.0);

//     // Ensure the result is positive and within the range [0, 360)
//     if avg_angle < 0.0 {
//         avg_angle + (360.0 * DEG_TO_RAD)
//     } else if avg_angle >= (360.0 * DEG_TO_RAD) {
//         avg_angle - (360.0 * DEG_TO_RAD)
//     } else {
//         avg_angle
//     }
// }
