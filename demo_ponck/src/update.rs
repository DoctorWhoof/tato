use crate::*;
use macroquad::input::*;
use tato::CollisionReaction;

pub fn move_player(game: &mut Game, world:&mut World) {
    // let speed_x = 30.0;
    // let speed_y = 20.0;

    // let max_speed_x = 120.0;
    // let max_speed_y = 120.0;

    // if is_key_down(KeyCode::Left) {
    //     game.paddle.vel.x -= speed_x;
    // } else if is_key_down(KeyCode::Right) {
    //     game.paddle.vel.x += speed_x;
    // } else if game.paddle.vel.x > 0.0 {
    //     game.paddle.vel.x -= speed_x;
    // } else if game.paddle.vel.x < 0.0 {
    //     game.paddle.vel.x += speed_x;
    // }

    // if is_key_down(KeyCode::Up) {
    //     game.paddle.vel.y -= speed_y;
    // } else if is_key_down(KeyCode::Down) {
    //     game.paddle.vel.y += speed_y;
    // } else if game.paddle.vel.y > 0.0 {
    //     game.paddle.vel.y -= speed_y;
    // } else if game.paddle.vel.y < 0.0 {
    //     game.paddle.vel.y += speed_y;
    // }

    // game.paddle.vel.x = game.paddle.vel.x.clamp(-max_speed_x, max_speed_x);
    // game.paddle.vel.y = game.paddle.vel.y.clamp(-max_speed_y, max_speed_y);
    
    let speed = 120.0;

    if is_key_down(KeyCode::Left) {
        game.paddle.vel.x = -speed;
    } else if is_key_down(KeyCode::Right) {
        game.paddle.vel.x = speed;
    } else {
        game.paddle.vel.x = 0.0;
    }

    if is_key_down(KeyCode::Up) {
        game.paddle.vel.y = -speed;
    } else if is_key_down(KeyCode::Down) {
        game.paddle.vel.y = speed;
    } else {
        game.paddle.vel.y = 0.0;
    }

    let min_x = 0.0;
    let max_x = 256.0;
    let min_y = 0.0;
    let max_y = 186.0;

    if let Some(col) = world.move_with_collision(game.paddle.id, game.paddle.vel, CollisionReaction::Slide) {
        game.paddle.vel = col.velocity;
        game.overlay.push( format!("{:.1?}", col) );
        // let hit = world.entity_add(10);
        // world.set_shape(hit, Shape::sprite_from_anim(TilesetID::Sprites, 1));
        // world.set_position(hit, col.pos);
        // world.set_render_offset(hit, -3, -3);
        // println!("{:?}", col);
    }

    if let Some(ent) = world.get_entity_mut(game.paddle.id) {
        if ent.pos.x > max_x {
            game.paddle.vel.x = 0.0;
            ent.pos.x = max_x;
        } else if ent.pos.x < min_x {
            game.paddle.vel.x = 0.0;
            ent.pos.x = min_x;
        }

        if ent.pos.y > max_y {
            game.paddle.vel.y = 0.0;
            ent.pos.y = max_y;
        } else if ent.pos.y < min_y {
            game.paddle.vel.y = 0.0;
            ent.pos.y = min_y;
        }
    }
}


pub fn move_puck(game:&mut Game, world:&mut World) {
    let max_speed = 120.0;
    game.puck.vel.clamp_to_length(max_speed);

    // let safety_speed = 180.0;
    // let deccelerate_rate = 15.0;
    // let elapsed = world.time_elapsed();
    // game.puck.vel.clamp_to_length(safety_speed);
    // if game.puck.vel.len() > max_speed {
    //     // println!("slow down!: {:?}", game.puck.vel.len());
    //     if game.puck.vel.x > 0.0 {
    //         game.puck.vel.x -= deccelerate_rate * elapsed;
    //     } else {
    //         game.puck.vel.x += deccelerate_rate * elapsed;
    //     }
    //     if game.puck.vel.y > 0.0 {
    //         game.puck.vel.y -= deccelerate_rate * elapsed;
    //     } else {
    //         game.puck.vel.y += deccelerate_rate * elapsed;
    //     }
    // }

    

    if let Some(col) = world.move_with_collision(game.puck.id, game.puck.vel, CollisionReaction::Bounce(1.0)){
        // Update puck velocity with "bounced" velocity
        game.puck.vel = col.velocity;

        // Destroy bricks!
        if col.colliding_entity == game.bricks {

            fn remove_brick(tilemap: &mut Tilemap, col:u16, row:u16) {
                tilemap.set_tile(col, row, Tile::default());
                tilemap.set_tile(col, row+1, Tile::default());
                if col % 2 == 0 {
                    tilemap.set_tile(col-1, row, Tile::default());
                    tilemap.set_tile(col-1, row+1, Tile::default());
                } else {
                    tilemap.set_tile(col+1, row, Tile::default());
                    tilemap.set_tile(col+1, row+1, Tile::default());
                }
            }

            if let Some(tile_col) = col.tile {           
                if let Some(ent) = world.get_entity_mut(game.bricks){
                    if let Shape::Bg { tileset_id, tilemap_id } = ent.shape {
                        let tilemap = world.renderer.get_tilemap_mut(tileset_id, tilemap_id);
                        remove_brick(tilemap, tile_col.col, tile_col.row);
                        game.score += 10;
                        // println!("removed {}, {}", tile_col.col, tile_col.row);
                    }
                }
            }
        }
    }

    // Out of bounds reset
    if let Some(ent) = world.get_entity_mut(game.puck.id) {
        if ent.pos.x < 0.0 || ent.pos.x > 256.0 || ent.pos.y < 0.0 || ent.pos.y > 190.0 {
            game.puck.vel = Vec2::default();
            ent.pos = game.puck.initial_pos;
        }
    }
}
