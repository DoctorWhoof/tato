mod specs;
pub use crate::specs::*;

use macroquad::prelude::*;
use spud::{Collider, Specs, World};

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

    let ent_point = world.add_entity(0);
    world.set_position(ent_point, 150.0, 96.0);
    let collider_point = Collider::from(spud::Vec2::zero());
    let collider_rect = Collider::from(spud::Rect{x:0.0, y:0.0, w:16.0, h:16.0});
    world.set_collider(ent_point, collider_point);

    let ent_rect_1 = world.add_entity(0);
    world.set_position(ent_rect_1, 160.0, 120.0);
    world.set_collider(ent_rect_1, Collider::from(spud::Rect{x:0.0, y:0.0, w:32.0, h:32.0}));

    let ent_sine_x = world.add_entity(0);
    world.set_position(ent_sine_x, 100.0, 60.0);
    world.set_collider(ent_sine_x, Collider::from(spud::Rect{x:0.0, y:0.0, w:32.0, h:16.0}));

    let ent_sine_y = world.add_entity(0);
    world.set_position(ent_sine_y, 40.0, 120.0);
    world.set_collider(ent_sine_y, Collider::from(spud::Rect{x:0.0, y:0.0, w:32.0, h:16.0}));

    let ent_sine = world.add_entity(0);
    world.set_position(ent_sine, 80.0, 120.0);
    world.set_collider(ent_sine, Collider::from(spud::Rect{x:0.0, y:0.0, w:32.0, h:16.0}));

    let ent_wall_top = world.add_entity(0);
    world.set_position(ent_wall_top, 0.0, 0.0);
    world.set_collider(ent_wall_top, Collider::from(spud::Rect{x:0.0, y:0.0, w:320.0, h:16.0}));

    let ent_wall_bottom = world.add_entity(0);
    world.set_position(ent_wall_bottom, 0.0, 224.0);
    world.set_collider(ent_wall_bottom, Collider::from(spud::Rect{x:0.0, y:0.0, w:320.0, h:16.0}));

    let ent_wall_left = world.add_entity(0);
    world.set_position(ent_wall_left, 0.0, 16.0);
    world.set_collider(ent_wall_left, Collider::from(spud::Rect{x:0.0, y:0.0, w:16.0, h:208.0}));

    let ent_wall_right = world.add_entity(0);
    world.set_position(ent_wall_right, 0.0, 16.0);
    world.set_collider(ent_wall_right, Collider::from(spud::Rect{x:304.0, y:0.0, w:16.0, h:208.0}));

    // main loop (infinite until "break")
    let time = std::time::Instant::now();
    let speed = 60.0;
    let mut vel = spud::Vec2::zero();
    loop {
        // Update
        world.start_frame(time.elapsed().as_secs_f32());
        if (is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Q)) || is_key_pressed(KeyCode::Escape) {
            break;
        }

        // Render scaling pre-calc
        let scale = (screen_height() / GameSpecs::RENDER_HEIGHT as f32).floor();
        let render_width = GameSpecs::RENDER_WIDTH as f32 * scale;
        let render_height = GameSpecs::RENDER_HEIGHT as f32 * scale;
        let draw_rect_x = (screen_width() - render_width) / 2.0;
        let draw_rect_y = (screen_height() - render_height) / 2.0;
                
        // Update

        if is_key_pressed(KeyCode::Key1){
            world.set_collider(ent_point, collider_point);
        } else if is_key_pressed(KeyCode::Key2){
            world.set_collider(ent_point, collider_rect);
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

        let sine_vel_x = spud::Vec2{x: oscillator.sin() * 60.0, y:0.0};
        world.move_with_collision(ent_sine_x, sine_vel_x, 0.0);

        let sine_vel_y = spud::Vec2{x: 0.0, y:oscillator.sin() * 60.0};
        world.move_with_collision(ent_sine_y, sine_vel_y, 0.0);

        let sine_vel = spud::Vec2{x: oscillator.sin() * 30.0, y:oscillator.cos() * 60.0};
        world.move_with_collision(ent_sine, sine_vel, 0.0);

        // Static colliders
        world.use_collider(ent_rect_1, spud::Vec2::zero());
        world.use_collider(ent_wall_top, spud::Vec2::zero());
        world.use_collider(ent_wall_bottom, spud::Vec2::zero());
        world.use_collider(ent_wall_left, spud::Vec2::zero());
        world.use_collider(ent_wall_right, spud::Vec2::zero());

        // Main Probe
        let collision = world.move_with_collision(ent_point, vel, 0.0);  //TODO: not &mut, simply set vel to col.vel?
        if let Some(col) = &collision { vel = col.velocity }

        world.framebuf.clear(spud::Color::gray_dark());
        world.render_frame();
        if let Some(col) = &collision {
            let line_len = 10.0;
            let x1 = col.pos.x + (col.normal.cos() * line_len);
            let y1 = col.pos.y - (col.normal.sin() * line_len);
            world.framebuf.draw_line(col.pos.x as i32, col.pos.y as i32, x1 as i32, y1 as i32, spud::Color::yellow());
            world.framebuf.draw_filled_rect(spud::Rect { x: col.pos.x as i32-1, y:col.pos.y as i32-1, w:3, h:3 }, spud::Color::red());
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
            for layer in &world.collision_layers {
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
