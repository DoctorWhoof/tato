use macroquad::{prelude::*, texture::Texture2D, window::next_frame};
use padstate::*;
use shared_game::*;

#[macroquad::main(window_conf)]
async fn main() {
    const W: u16 = 256;
    const H: u16 = 196;

    let mut vid = tato_video::VideoChip::new(W, H);
    let mut scene = shared_game::Scene::A(CameraScrolling::new(&mut vid));
    let mut pad = padstate::DPad::new();
    let mut render_image = Image::gen_image_color(W.into(), H.into(), BLACK);
    let render_texture = Texture2D::from_image(&render_image);
    render_texture.set_filter(FilterMode::Nearest);

    loop {
        // Input
        pad.copy_current_to_previous_state();
        if is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Q) {
            break;
        }
        if is_key_down(KeyCode::Left) {
            pad.set_state(Button::Left, true);
        } else {
            pad.set_state(Button::Left, false)
        }
        if is_key_down(KeyCode::Right) {
            pad.set_state(Button::Right, true);
        } else {
            pad.set_state(Button::Right, false)
        }
        if is_key_down(KeyCode::Up) {
            pad.set_state(Button::Up, true);
        } else {
            pad.set_state(Button::Up, false)
        }
        if is_key_down(KeyCode::Down) {
            pad.set_state(Button::Down, true);
        } else {
            pad.set_state(Button::Down, false)
        }

        // Update
        let state = AppState {
            pad,
            time: macroquad::time::get_time(),
            elapsed: 1.0 / 120.0,
        };
        // println!("{:.1?}", macroquad::time::get_fps());
        let mode_request = match &mut scene {
            Scene::A(scn) => scn.update(&mut vid, state),
            Scene::B(scn) => scn.update(&mut vid, state),
            Scene::C(scn) => scn.update(&mut vid, state),
        };

        // Copy from framebuffer to macroquad texture
        for (pixel, coord) in vid.iter_pixels() {
            render_image.set_pixel(
                coord.x as u32,
                coord.y as u32,
                Color::from_rgba(pixel.r, pixel.g, pixel.b, 255),
            )
        }

        // Scenes request a new scene if their return is Some(mode).
        // Processed after copying pixels for this frame.
        if let Some(mode) = mode_request {
            vid.reset_all();
            match mode {
                Mode::A => scene = Scene::A(CameraScrolling::new(&mut vid)),
                Mode::B => scene = Scene::B(FixedCamera::new(&mut vid)),
                Mode::C => scene = Scene::C(MinimalScene::new(&mut vid)),
            }
        }

        clear_background(BLACK);
        render_texture.update(&render_image);
        let scale = screen_height() / H as f32;
        let w = W as f32 * scale;
        let h = H as f32 * scale;
        let draw_rect_x = (screen_width() - w) / 2.0;
        let draw_rect_y = (screen_height() - h) / 2.0;
        draw_texture_ex(
            &render_texture,
            draw_rect_x,
            draw_rect_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                source: None,
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );

        next_frame().await
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Videochip".into(),
        fullscreen: false,
        high_dpi: true,
        sample_count: 0,
        window_resizable: true,
        window_width: 288 * 3,
        window_height: 216 * 3,
        ..Default::default()
    }
}
