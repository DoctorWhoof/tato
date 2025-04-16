use raylib::prelude::*;
use std::time::Instant;
use tato::{audio::*, backend::*, pad::*, video::*};

pub struct Backend {
    target_fps: f64,
    pixels: [u8; 240 * 180 * 4],
    frame_start_time: Instant,
    render_texture: Texture2D,
    pad: AnaloguePad,
    rl_handle: RaylibHandle,
    rl_thread: RaylibThread,
    // audio_stream: AudioStream<'a>,
}

impl TatoBackend for Backend {
    fn new_window(video_chip: Option<&VideoChip>, audio_chip: Option<&AudioChip>) -> Self {
        // let audio_chip = audio_chip.unwrap();
        let video_chip = video_chip.unwrap();

        let w = video_chip.width() as i32;
        let h = video_chip.height() as i32;

        let (mut rl_handle, rl_thread) = raylib::init()
            .size(w * 3, h * 3)
            .title("Tato Videochip")
            .vsync()
            .resizable()
            .build();

        let target_fps = 60.0;
        rl_handle.set_target_fps(target_fps as u32);

        // Create image and texture for rendering
        let render_image = Image::gen_image_color(w, h, Color::BLACK);
        let render_texture = rl_handle
            .load_texture_from_image(&rl_thread, &render_image)
            .unwrap();

        unsafe {
            raylib::ffi::SetConfigFlags(raylib::ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32);
            // Disable ESC to close window. "Cmd + Q" still works!
            raylib::ffi::SetExitKey(raylib::ffi::KeyboardKey::KEY_NULL as i32);
        }

        // let audio = raylib::core::audio::RaylibAudio::init_audio_device().unwrap();
        // let audio_stream = audio.new_audio_stream(
        //     audio_chip.sample_rate,
        //     16,
        //     1
        // );
        Self {
            target_fps,
            frame_start_time: Instant::now(),
            pixels: std::array::from_fn(|_| 0),
            render_texture,
            pad: AnaloguePad::default(),
            rl_handle,
            rl_thread,
            // audio_stream
        }
    }

    fn frame_start(&mut self, _vid: &VideoChip) {
        // Timing
        self.frame_start_time = Instant::now();

        // Ensures gamepad can detect button changes
        self.pad.copy_current_to_previous_state();

        // Handle keyboard input
        if self.rl_handle.is_key_down(KeyboardKey::KEY_LEFT) {
            self.pad.set_button(Button::Left, true);
        } else {
            self.pad.set_button(Button::Left, false);
        }

        if self.rl_handle.is_key_down(KeyboardKey::KEY_RIGHT) {
            self.pad.set_button(Button::Right, true);
        } else {
            self.pad.set_button(Button::Right, false);
        }

        if self.rl_handle.is_key_down(KeyboardKey::KEY_UP) {
            self.pad.set_button(Button::Up, true);
        } else {
            self.pad.set_button(Button::Up, false);
        }

        if self.rl_handle.is_key_down(KeyboardKey::KEY_DOWN) {
            self.pad.set_button(Button::Down, true);
        } else {
            self.pad.set_button(Button::Down, false);
        }

        if self.rl_handle.is_key_down(KeyboardKey::KEY_ESCAPE) {
            self.pad.set_button(Button::Menu, true);
        } else {
            self.pad.set_button(Button::Menu, false);
        }

        if self.rl_handle.is_key_down(KeyboardKey::KEY_ENTER) {
            self.pad.set_button(Button::Start, true);
        } else {
            self.pad.set_button(Button::Start, false);
        }
    }

    fn frame_update(&mut self, _vid: &VideoChip) {
        // Not necessary for this backend
    }

    fn frame_finish(&mut self, vid: &VideoChip) {
        // Copy from framebuffer to raylib texture
        for (color, coords) in vid.iter_pixels() {
            let i = ((coords.y as usize * 240) + coords.x as usize) * 4;
            self.pixels[i] = color.r;
            self.pixels[i + 1] = color.g;
            self.pixels[i + 2] = color.b;
            self.pixels[i + 3] = 255;
        }
        self.render_texture.update_texture(&self.pixels).unwrap();

        // Calculate rect with correct aspect ratio with integer scaling
        let screen_width = self.rl_handle.get_screen_width();
        let screen_height = self.rl_handle.get_screen_height();

        let scale = (screen_height as f32 / vid.height() as f32).floor() as i32;
        let w = vid.width() as i32 * scale;
        let h = vid.height() as i32 * scale;
        let draw_rect_x = (screen_width - w) / 2;
        let draw_rect_y = (screen_height - h) / 2;

        // // Get update time before actually drawing
        // println!(
        //     "fps:{:.1}, elapsed:{:.1}, update:{:.1}",
        //     self.rl_handle.get_fps(),
        //     self.rl_handle.get_frame_time() * 1000.0,
        //     self.frame_start_time.elapsed().as_secs_f64() * 1000.0
        // );

        // Present frame
        {
            let mut d = self.rl_handle.begin_drawing(&self.rl_thread);
            d.clear_background(Color::BLACK);
            // d.draw_texture(&self.render_texture, 0, 0, Color::WHITE);
            d.draw_texture_ex(
                &self.render_texture,
                Vector2::new(draw_rect_x as f32, draw_rect_y as f32),
                0.0,
                scale as f32,
                Color::WHITE,
            );
        }
    }

    fn elapsed(&self) -> f64 {
        1.0 / self.target_fps
    }

    fn time(&self) -> f64 {
        self.rl_handle.get_time() as f64
    }

    fn quit_requested(&self) -> bool {
        self.rl_handle.window_should_close()
    }

    // Input
    fn gamepad(&self) -> AnaloguePad {
        self.pad
    }

    // Audio
    fn audio_update_buffer(&mut self, _audio: &AudioChip) {
        // self.
    }
}
