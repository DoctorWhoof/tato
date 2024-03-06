mod specs;
pub use crate::specs::*;

use macroquad::prelude::*;
use tato::{Collider, CollisionReaction, Specs, World};

pub type GameWorld = World<GameSpecs, TilesetID, PaletteID>;

#[macroquad::main(window_conf)]
async fn main() {
    // macroquad init
    let mut img = Image::gen_image_color(
        GameSpecs::RENDER_WIDTH as u16,
        GameSpecs::RENDER_HEIGHT as u16,
        BLACK,
    );
    let render_texture = Texture2D::from_image(&img);
    render_texture.set_filter(FilterMode::Nearest);

    // Spud init
    let mut world: GameWorld = World::new();
    world.debug_colliders = true;   
    world.debug_pivot = true;   

    let ent_main = world.add_entity(0);
    let initial_position = tato::Vec2{x:160.0, y:100.0};
    world.set_position(ent_main, initial_position);
    let collider_point = Collider::from(tato::Vec2{x:0.0, y:0.0});
    let collider_rect = Collider::from(tato::Rect{x:-8.0, y:-8.0, w:16.0, h:16.0});
    world.add_collider(ent_main, collider_rect);

    let ent_rect_1 = world.add_entity(0);
    world.set_position(ent_rect_1, tato::Vec2::new(160.0, 120.0));
    world.add_collider(ent_rect_1, Collider::from(tato::Rect{x:0.0, y:0.0, w:32.0, h:32.0}));

    let ent_sine_x = world.add_entity(0);
    world.set_position(ent_sine_x, tato::Vec2::new(100.0, 60.0));
    world.add_collider(ent_sine_x, Collider::from(tato::Rect{x:0.0, y:0.0, w:32.0, h:16.0}));

    let ent_sine_y = world.add_entity(0);
    world.set_position(ent_sine_y, tato::Vec2::new(40.0, 120.0));
    world.add_collider(ent_sine_y, Collider::from(tato::Rect{x:0.0, y:0.0, w:32.0, h:16.0}));

    let ent_sine = world.add_entity(0);
    world.set_position(ent_sine, tato::Vec2::new(80.0, 120.0));
    world.add_collider(ent_sine, Collider::from(tato::Rect{x:0.0, y:0.0, w:32.0, h:16.0}));

    let ent_wall_top = world.add_entity(0);
    world.set_position(ent_wall_top, tato::Vec2::new(0.0, 0.0));
    world.add_collider(ent_wall_top, Collider::from(tato::Rect{x:0.0, y:0.0, w:320.0, h:16.0}));

    let ent_wall_bottom = world.add_entity(0);
    world.set_position(ent_wall_bottom, tato::Vec2::new(0.0, 224.0));
    world.add_collider(ent_wall_bottom, Collider::from(tato::Rect{x:0.0, y:0.0, w:320.0, h:16.0}));

    let ent_wall_left = world.add_entity(0);
    world.set_position(ent_wall_left, tato::Vec2::new(0.0, 16.0));
    world.add_collider(ent_wall_left, Collider::from(tato::Rect{x:0.0, y:0.0, w:16.0, h:208.0}));

    let ent_wall_right = world.add_entity(0);
    world.set_position(ent_wall_right, tato::Vec2::new(304.0, 16.0));
    world.add_collider(ent_wall_right, Collider::from(tato::Rect{x:0.0, y:0.0, w:16.0, h:208.0}));

    // main loop
    let time = std::time::Instant::now();
    let speed = 120.0;
    let mut vel = tato::Vec2::zero();
    loop {
        // Update
        world.start_frame(time.elapsed().as_secs_f32());
        if is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Q) {
            break;
        }

        if is_key_pressed(KeyCode::Escape){
            world.set_position(ent_main, initial_position)
        }

        // Render scaling pre-calc
        let scale = (screen_height() / GameSpecs::RENDER_HEIGHT as f32).floor();
        let render_width = GameSpecs::RENDER_WIDTH as f32 * scale;
        let render_height = GameSpecs::RENDER_HEIGHT as f32 * scale;
        let draw_rect_x = (screen_width() - render_width) / 2.0;
        let draw_rect_y = (screen_height() - render_height) / 2.0;
                
        // Update
        if is_key_pressed(KeyCode::Key1){
            world.add_collider(ent_main, collider_point);
        } else if is_key_pressed(KeyCode::Key2){
            world.add_collider(ent_main, collider_rect);
        }

        if is_key_down(KeyCode::Up) {
            vel.y = -speed
        }else if is_key_down(KeyCode::Down) {
            vel.y = speed
        } else {
            vel.y = 0.0
        }

        if is_key_down(KeyCode::Left) {
            vel.x = -speed
        } else if is_key_down(KeyCode::Right) {
            vel.x = speed
        } else {
            vel.x = 0.0
        }

        // Moving colliders
        let oscillator = world.time() * 2.0;
        
        let sine_vel_x = tato::Vec2{x: oscillator.sin() * 60.0, y:0.0};
        world.move_with_collision(ent_sine_x, sine_vel_x, CollisionReaction::None);

        let sine_vel_y = tato::Vec2{x: 0.0, y:oscillator.sin() * 60.0};
        world.move_with_collision(ent_sine_y, sine_vel_y, CollisionReaction::None);

        let sine_vel = tato::Vec2{x: oscillator.sin() * 30.0, y:oscillator.cos() * 60.0};
        world.move_with_collision(ent_sine, sine_vel, CollisionReaction::None);

        // Static colliders
        world.use_static_collider(ent_rect_1);
        world.use_static_collider(ent_wall_top);
        world.use_static_collider(ent_wall_bottom);
        world.use_static_collider(ent_wall_left);
        world.use_static_collider(ent_wall_right);

        // Main Probe
        let collision = world.move_with_collision(ent_main, vel, CollisionReaction::Slide);  //TODO: not &mut, simply set vel to col.vel?
        if let Some(col) = &collision {
            // println!("{:?}", col);
            vel = col.velocity
        }

        world.framebuf.clear(tato::Color::gray_dark());
        world.render_frame();
        if let Some(col) = &collision {
            let pos = world.get_position(ent_main);
            let line_len = 10.0;
            let x1 = pos.x + (col.normal.x * line_len);
            let y1 = pos.y + (col.normal.y * line_len);
            world.framebuf.draw_line(pos.x as i32, pos.y as i32, x1 as i32, y1 as i32, tato::Color::yellow());
            world.framebuf.draw_filled_rect(tato::Rect { x: pos.x as i32-1, y:pos.y as i32-1, w:3, h:3 }, tato::Color::red());
        }

        // Copy from framebuffer to macroquad texture
        let source = world.framebuf.pixels();
        let width = GameSpecs::RENDER_WIDTH;
        for y in 0..GameSpecs::RENDER_HEIGHT {
            for x in 0..GameSpecs::RENDER_WIDTH {
                let source_index = (y * width) + x;
                let color = source[source_index];
                img.set_pixel(
                    x as u32,
                    y as u32,
                    Color::from_rgba(color.r, color.g, color.b, color.a),
                )
            }
        }

        // Render texture to screen
        clear_background(BLACK);
        render_texture.update(&img);
        draw_texture_ex(
            &render_texture,
            draw_rect_x,
            draw_rect_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(render_width, render_height)),
                source: None,
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );

        {   // Debug text
            let mut i = 10.0;
            draw_text(
                format!("Vel: {:.2?}", vel).as_str(),
                10.0, i, 16.0, WHITE
            );

            i += 24.0;
            for layer in world.get_collision_layers() {
                i += 8.0;
                for col in layer {
                    i += 16.0;
                    draw_text(
                        format!("Collider: {:.2?}", col).as_str(),
                        10.0, i, 16.0, WHITE
                    );
                }
            }

            i += 24.0;
            if let Some(col) = &collision {
                draw_text(
                    format!("Collision: {:.2?}", col).as_str(),
                    10.0, i, 16.0, WHITE
                );
            }
        }

        // Finish (calculate timings)
        world.finish_frame(time.elapsed().as_secs_f32());
        next_frame().await;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Collision Test".into(),
        fullscreen: false,
        high_dpi: true,
        sample_count: 0,
        window_resizable: true,
        window_width: 320 * 3,
        window_height: 240 * 3,
        ..Default::default()
    }
}
