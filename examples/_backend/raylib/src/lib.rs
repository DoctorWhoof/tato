pub use raylib;
use raylib::prelude::*;
use tato::{Tato, prelude::*};

// pub struct RaylibBackend<const PIXEL_COUNT: usize> {
pub struct RaylibBackend {
    pub ray: RaylibHandle,
    pub display_debug: bool,
    pub display_debug_scale: i32,
    thread: RaylibThread,
    pixels: Vec<u8>,
    debug_pixels: Vec<Vec<u8>>,
    render_texture: Texture2D,
    debug_texture: Vec<Texture2D>,
}

impl RaylibBackend {
    pub fn new(tato: &Tato, target_fps: f32) -> Self {
        let w = tato.video.width() as i32;
        let h = tato.video.height() as i32;
        let (mut ray, thread) = raylib::init()
            .log_level(TraceLogLevel::LOG_WARNING)
            .size(w * 5, h * 3)
            .title("Tato Demo")
            .vsync()
            .resizable()
            .build();
        // Config raylib
        ray.set_target_fps(target_fps as u32);
        unsafe {
            raylib::ffi::SetConfigFlags(raylib::ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32);
            // Disable ESC to close window. "Cmd + Q" still works!
            raylib::ffi::SetExitKey(raylib::ffi::KeyboardKey::KEY_NULL as i32);
        }
        // Create texture for rendering
        let pixels = vec![0u8; w as usize * h as usize * 4];
        let render_texture = {
            let render_image = Image::gen_image_color(w, h, Color::BLACK);
            ray.load_texture_from_image(&thread, &render_image).unwrap()
        };
        // Create textures for debug display
        let tiles_per_row = (TILE_COUNT as f64).sqrt().ceil() as usize;
        let debug_w = tiles_per_row * TILE_SIZE as usize;
        let debug_h = ((TILE_COUNT + tiles_per_row - 1) / tiles_per_row) * TILE_SIZE as usize;
        let debug_pixels = vec![vec![0u8; debug_w * debug_h * 4]; tato.banks.len()];
        let debug_image = Image::gen_image_color(debug_w as i32, debug_h as i32, Color::BLACK);
        let debug_texture = (0..tato.banks.len())
            .map(|_| ray.load_texture_from_image(&thread, &debug_image).unwrap())
            .collect();
        // Build struct
        Self {
            ray,
            display_debug: true,
            display_debug_scale: 1,
            thread,
            pixels,
            debug_pixels,
            render_texture,
            debug_texture,
        }
    }

    pub fn update_gamepad(&self, pad: &mut AnaloguePad) {
        // Ensures gamepad can detect button changes
        pad.copy_current_to_previous_state();

        // Handle keyboard input
        if self.ray.is_key_down(KeyboardKey::KEY_LEFT) {
            pad.set_button(Button::Left, true);
        } else {
            pad.set_button(Button::Left, false);
        }

        if self.ray.is_key_down(KeyboardKey::KEY_RIGHT) {
            pad.set_button(Button::Right, true);
        } else {
            pad.set_button(Button::Right, false);
        }

        if self.ray.is_key_down(KeyboardKey::KEY_UP) {
            pad.set_button(Button::Up, true);
        } else {
            pad.set_button(Button::Up, false);
        }

        if self.ray.is_key_down(KeyboardKey::KEY_DOWN) {
            pad.set_button(Button::Down, true);
        } else {
            pad.set_button(Button::Down, false);
        }

        if self.ray.is_key_down(KeyboardKey::KEY_ESCAPE) {
            pad.set_button(Button::Menu, true);
        } else {
            pad.set_button(Button::Menu, false);
        }

        if self.ray.is_key_down(KeyboardKey::KEY_ENTER) {
            pad.set_button(Button::Start, true);
        } else {
            pad.set_button(Button::Start, false);
        }

        if self.ray.is_key_down(KeyboardKey::KEY_Z) {
            pad.set_button(Button::A, true);
        } else {
            pad.set_button(Button::A, false);
        }

        if self.ray.is_key_down(KeyboardKey::KEY_ONE) {
            pad.set_button(Button::LeftShoulder, true);
        } else {
            pad.set_button(Button::LeftShoulder, false);
        }
    }

    pub fn render(&mut self, t: &mut Tato) {
        let time = std::time::Instant::now();

        if self.ray.is_key_pressed(KeyboardKey::KEY_TAB) {
            self.display_debug = !self.display_debug
        }
        if self.ray.is_key_pressed(KeyboardKey::KEY_EQUAL) {
            self.display_debug_scale += 1
        }
        if self.ray.is_key_pressed(KeyboardKey::KEY_MINUS) {
            if self.display_debug_scale > 1 {
                self.display_debug_scale -= 1;
            }
        }

        // Copy from framebuffer to raylib texture
        for (color, coords) in t.video.iter_pixels(&t.get_video_banks(), &[&t.bg]) {
            let i = ((coords.y as usize * t.video.width() as usize) + coords.x as usize) * 4;
            self.pixels[i] = color.r;
            self.pixels[i + 1] = color.g;
            self.pixels[i + 2] = color.b;
            self.pixels[i + 3] = color.a;
        }
        self.render_texture.update_texture(&self.pixels).unwrap();

        t.update_time_acc.push(time.elapsed().as_secs_f64() * 1000.0);
        println!("iter time: {:.2} ms", t.update_time_acc.average());

        let tiles_per_row = (TILE_COUNT as f64).sqrt().ceil() as usize;
        let debug_w = (tiles_per_row * TILE_SIZE as usize) as i32;
        let debug_h =
            (((TILE_COUNT + tiles_per_row - 1) / tiles_per_row) * TILE_SIZE as usize) as i32;

        // Calculate rect with correct aspect ratio with integer scaling
        let screen_width = self.ray.get_screen_width();
        let screen_height = self.ray.get_screen_height();

        let scale = (screen_height as f32 / t.video.height() as f32).floor() as i32;
        let w = t.video.width() as i32 * scale;
        let h = t.video.height() as i32 * scale;
        let draw_rect_x = (screen_width - w) / 2;
        let draw_rect_y = (screen_height - h) / 2;

        // Present pixels
        let mut canvas = self.ray.begin_drawing(&self.thread);
        let bg_color = Color::new(64, 64, 64, 255);

        canvas.clear_background(bg_color);
        canvas.draw_texture_ex(
            &self.render_texture,
            Vector2::new(draw_rect_x as f32, draw_rect_y as f32),
            0.0,
            scale as f32,
            Color::WHITE,
        );

        // Copy tile pixels to debug texture
        if self.display_debug {
            let color_bg = Color::new(0, 0, 0, 100);
            let font_size = 8;
            let rect_bg = Rect {
                x: screen_width - (debug_w as i32 * self.display_debug_scale) - 8,
                y: font_size,
                w: debug_w * self.display_debug_scale,
                h: screen_height - font_size - font_size,
            };
            canvas.draw_rectangle(rect_bg.x, rect_bg.y, rect_bg.w, rect_bg.h, color_bg);
            let mut layout = Frame::new(rect_bg);
            // Reset on every loop since I may change it along the way!
            layout.set_gap(1);
            layout.set_margin(1);
            layout.set_scale(self.display_debug_scale as f32);
            layout.fitting = Fitting::Clamp;

            for bank_index in 0..BANK_COUNT {
                let bank = &t.banks[bank_index];

                // acquire tile pixels.
                for tile_index in 0..TILE_COUNT {
                    let tile_x = tile_index % tiles_per_row;
                    let tile_y = tile_index / tiles_per_row;

                    for y in 0..TILE_SIZE as usize {
                        for x in 0..TILE_SIZE as usize {
                            let color_index = bank.tiles[tile_index].get_pixel(x as u8, y as u8);
                            let gray_value = color_index * 63; // Map 0-4 to 0-252

                            let pixel_x = tile_x * TILE_SIZE as usize + x;
                            let pixel_y = tile_y * TILE_SIZE as usize + y;
                            let pixel_offset = (pixel_y * debug_w as usize + pixel_x) * 4;

                            // Set RGBA values
                            self.debug_pixels[bank_index][pixel_offset] = gray_value; // R
                            self.debug_pixels[bank_index][pixel_offset + 1] = gray_value; // G
                            self.debug_pixels[bank_index][pixel_offset + 2] = gray_value; // B
                            self.debug_pixels[bank_index][pixel_offset + 3] = 255; // A
                        }
                    }
                }

                // Label
                layout.push_edge(Edge::Top, font_size / self.display_debug_scale, |frame| {
                    let rect = frame.rect();
                    let text = format!("bank {}", bank_index);
                    canvas.draw_text(&text, rect.x + 1, rect.y, font_size, Color::WHITE);
                });

                // Color swatches
                layout.push_edge(Edge::Top, 8, |frame| {
                    // draw bg
                    let rect = frame.rect();
                    canvas.draw_rectangle(rect.x, rect.y, rect.w, rect.h, color_bg);
                    let swatch_w = frame.divide_width(COLORS_PER_PALETTE as u32);
                    for c in 0..COLORS_PER_PALETTE as usize {
                        let color = bank.palette[c];
                        frame.push_edge(Edge::Left, swatch_w, |swatch| {
                            let rect = swatch.rect();
                            canvas.draw_rectangle(rect.x, rect.y, rect.w, rect.h, rl_color(color));
                        });
                    }
                });

                // Subpalettes
                let columns = 4;
                let rows = (bank.sub_palette_count() as f32 / columns as f32).ceil() as u32;
                let frame_h = (rows as i32 * 8) + 4;
                layout.push_edge(Edge::Top, frame_h, |frame| {
                    let column_w = frame.divide_width(columns);
                    for column in 0..columns {
                        frame.push_edge(Edge::Left, column_w, |frame_column| {
                            frame_column.set_gap(0);
                            frame_column.set_margin(1);
                            // draw bg
                            let rect = frame_column.rect();
                            canvas.draw_rectangle(rect.x, rect.y, rect.w, rect.h, color_bg);
                            // draw each row
                            let row_h = frame_column.divide_height(rows);
                            for row in 0..rows {
                                frame_column.push_edge(Edge::Top, row_h, |frame_row| {
                                    frame_row.set_gap(0);
                                    frame_row.set_margin(1);
                                    let subp_index =
                                        ((row * COLORS_PER_TILE as u32) + column) as usize;
                                    let subp = &bank.sub_palettes[subp_index];
                                    // draw each swatch
                                    let swatch_w = frame_row.divide_width(COLORS_PER_TILE as u32);
                                    for n in 0..COLORS_PER_TILE as usize {
                                        frame_row.push_edge(Edge::Left, swatch_w, |swatch| {
                                            let r = swatch.rect();
                                            let color_index = subp[n].0 as usize;
                                            let color = bank.palette[color_index];
                                            canvas.draw_rectangle(
                                                r.x,
                                                r.y,
                                                r.w,
                                                r.h,
                                                rl_color(color),
                                            );
                                        });
                                    }
                                });
                            }
                        });
                    }
                });

                // tiles
                layout.push_edge(Edge::Top, debug_h, |frame_tiles| {
                    self.debug_texture[bank_index]
                        .update_texture(&self.debug_pixels[bank_index])
                        .unwrap();
                    let r = frame_tiles.rect();
                    canvas.draw_texture_ex(
                        &self.debug_texture[bank_index],
                        Vector2::new(r.x as f32, r.y as f32),
                        0.0,
                        self.display_debug_scale as f32,
                        Color::WHITE,
                    );
                });
            }
        }
    }
}

fn rl_color(color: Color12Bit) -> Color {
    Color::new(
        ((color.r() as u16 * 255) / 7) as u8,
        ((color.g() as u16 * 255) / 7) as u8,
        ((color.b() as u16 * 255) / 7) as u8,
        ((color.a() as u16 * 255) / 7) as u8,
    )
}
