use macroquad::prelude::*;
use tato::{Specs, World};

#[macroquad::main(window_conf)]
async fn main() {
    let specs = Specs {
        render_width: 320,
        render_height: 240,
        atlas_width: 128,
        atlas_height: 128,
        tile_width: 8,
        tile_height: 8,
        colors_per_palette: 16,
    };

    // macroquad init
    let mut img = Image::gen_image_color( specs.render_width, specs.render_height, BLACK);
    let render_texture = Texture2D::from_image(&img);
    render_texture.set_filter(FilterMode::Nearest);

    // Spud init
    let mut world = World::new(specs);
    
    // *********************** Add entities here! *********************** 
    
    // main loop
    let time = std::time::Instant::now();
    loop {
        world.start_frame(time.elapsed().as_secs_f32());
        
        // Input
        if is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Q) {
            break;
        }

        // Render scaling pre-calc
        let scale = (screen_height() / specs.render_height as f32).floor();
        let render_width = specs.render_width as f32 * scale;
        let render_height = specs.render_height as f32 * scale;
        let draw_rect_x = (screen_width() - render_width) / 2.0;
        let draw_rect_y = (screen_height() - render_height) / 2.0;
                
        // *********************** Entity Update goes here *********************** 

        // Render
        world.framebuf.clear(tato::Color24::gray_dark());
        world.render_frame();

        // Copy from framebuffer to macroquad texture
        let source = world.framebuf.pixels();
        let width = specs.render_width;
        for y in 0..specs.render_height {
            for x in 0..specs.render_width {
                let source_index = (y * width) + x;
                let color = source[source_index as usize];
                img.set_pixel(
                    x as u32,
                    y as u32,
                    Color24::from_rgba(color.r, color.g, color.b, color.a),
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
