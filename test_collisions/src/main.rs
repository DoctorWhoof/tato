mod specs;
pub use crate::specs::*;

use macroquad::prelude::*;
use spud::{Collider, Specs, Vec2, World};

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

    let ent_point = world.insert_entity(0);
    world.set_position(ent_point, 150.0, 100.0);
    world.set_collider(ent_point, Collider::new_point(0, 0));

    let ent_rect_1 = world.insert_entity(0);
    world.set_position(ent_rect_1, 160.0, 120.0);
    world.set_collider(ent_rect_1, Collider::new_rect(0.0, 0.0, 32.0, 32.0, 0, 0));

    // main loop (infinite until "break")
    let time = std::time::Instant::now();
    let speed = 60.0;
    let mut vel = Vec2::zero();
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
        if is_key_down(KeyCode::Up) {
            vel.y = -speed
        }else if is_key_down(KeyCode::Down) {
            vel.y = speed
        }else {
            vel.y = 0.0
        }

        if is_key_down(KeyCode::Left) {
            vel.x = -speed
        } else if is_key_down(KeyCode::Right) {
            vel.x = speed
        } else {
            vel.x = 0.0
        }

        let col = world.move_and_collide(ent_point, &mut vel, ent_rect_1, Vec2::zero());

        // Render
        
        // if let Some(entity) = world.get_entity(ent_point) {
        //     world.framebuf.draw_pixel(entity.pos.x as usize, entity.pos.y as usize, spud::Color::white());
        // }




        // if let Some(entity) = world.get_entity(ent_rect_1) {
        //     if let Some(collider) = &entity.collider {
        //         if let ColliderKind::Rect(mut rect) = collider.kind {
        //             rect.x += entity.pos.x;
        //             rect.y += entity.pos.y;
        //             if let Some(col) = &col {
        //                 let line_len = 10.0;
        //                 let x1 = col.point.x + (col.normal.cos() * line_len);
        //                 let y1 = col.point.y - (col.normal.sin() * line_len);
        //                 world.framebuf.draw_line(col.point.x as i32, col.point.y as i32, x1 as i32, y1 as i32, spud::Color::yellow());
        //                 world.framebuf.draw_rect(rect.to_i32(), spud::Color::red());
        //                 world.framebuf.draw_filled_rect(spud::Rect { x: col.point.x as i32-1, y:col.point.y as i32-1, w:3, h:3 }, spud::Color::red());
        //             } else {
        //                 world.framebuf.draw_rect(rect.to_i32(), spud::Color::orange());
        //             };
        //         }            
        //     }
        // }

        world.framebuf.clear(spud::Color::gray_dark());
        world.render_frame();
        if let Some(col) = &col {
            let line_len = 10.0;
            let x1 = col.point.x + (col.normal.cos() * line_len);
            let y1 = col.point.y - (col.normal.sin() * line_len);
            world.framebuf.draw_line(col.point.x as i32, col.point.y as i32, x1 as i32, y1 as i32, spud::Color::yellow());
            world.framebuf.draw_filled_rect(spud::Rect { x: col.point.x as i32-1, y:col.point.y as i32-1, w:3, h:3 }, spud::Color::red());
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
        if let Some(col) = &col {
            draw_text(
                format!("Normal: {}", col.normal).as_str(),
                10.0, 10.0, 16.0, WHITE
            );
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
