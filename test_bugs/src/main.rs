
mod game;
mod actors; 
mod specs;

pub use game::*;
pub use actors::*;
pub use specs::*;

use tato::{Atlas, World};
use macroquad::{color::*, input::*, math::*, text::draw_text, texture::*, window::*};

#[macroquad::main(window_conf)]
async fn main() {
    // macroquad init
    let mut img = Image::gen_image_color( SPECS.render_width, SPECS.render_height, BLACK);
    let render_texture = Texture2D::from_image(&img);
    render_texture.set_filter(FilterMode::Nearest);

    // Game Init 
    let mut game = Game::new(SPECS);
    
    // Main loop
    let time = std::time::Instant::now();
    loop {
        game.world.start_frame(time.elapsed().as_secs_f32());
        
        // Global Input
        if is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Q) { break; }
        if is_key_pressed(KeyCode::A) { game.world.debug_atlas = !game.world.debug_atlas }
        if is_key_pressed(KeyCode::W) { game.world.debug_pivot = !game.world.debug_pivot }

        // Render scaling pre-calc
        let scale = (screen_height() / SPECS.render_height as f32).floor();
        let render_width = SPECS.render_width as f32 * scale;
        let render_height = SPECS.render_height as f32 * scale;
        let draw_rect_x = (screen_width() - render_width) / 2.0;
        let draw_rect_y = (screen_height() - render_height) / 2.0;
                
        // Update game and render entities 
        game.update();
        game.world.render_frame();

        // Copy from framebuffer to macroquad texture
        let source = game.world.framebuf.pixels();
        let width = SPECS.render_width;
        for y in 0..SPECS.render_height {
            for x in 0..SPECS.render_width {
                let source_index = (y * width) + x;
                let color = source[source_index as usize];
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

        // Overlay
        let mut i = 10.0;
        draw_text( format!("FPS: {:.1}", 1.0 / game.world.time_elapsed()).as_str(), 16.0, i as f32, 16.0, WHITE);
        i += 16.0;
        draw_text( format!("Entity count: {}", game.world.entities().len()).as_str(), 16.0, i as f32, 16.0, WHITE);
        i += 16.0;
        draw_text( format!("Update time: {:.2}", game.world.time_update() * 1000.0).as_str(), 16.0, i as f32, 16.0, WHITE);
        i += 16.0;
        for bullet in game.bullets.iter() {
            draw_text(
                format!("{:?}", bullet).as_str(),
                16.0, i as f32, 16.0, WHITE
            );
            i += 16.0;
        }

        // Finish (calculate timings)
        game.world.finish_frame(time.elapsed().as_secs_f32());
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
        window_width: 256 * 3,
        window_height: 192 * 3,
        ..Default::default()
    }
}
