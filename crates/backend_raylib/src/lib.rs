use std::vec;

pub use raylib;
use raylib::prelude::*;
use tato::{Tato, prelude::*};

pub use tato;

// pub struct RaylibBackend<const PIXEL_COUNT: usize> {
pub struct RaylibBackend {
    pub bg_color: Color,
    pub ray: RaylibHandle,
    pub display_debug: bool,
    pub display_debug_scale: i32,
    thread: RaylibThread,
    pixels: Vec<u8>,
    debug_pixels: Vec<Vec<u8>>,
    render_texture: Texture2D,
    debug_texture: Vec<Texture2D>,
    font: Font,
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

        // Embed Font file
        let font_data = include_bytes!("font.ttf");
        let font = ray.load_font_from_memory(&thread, ".ttf", font_data, 32, None).unwrap();

        // Create texture for rendering
        let pixels = vec![0u8; w as usize * h as usize * 4];
        let render_texture = {
            let render_image = Image::gen_image_color(w, h, Color::BLACK);
            ray.load_texture_from_image(&thread, &render_image).unwrap()
        };

        // Pre-populate textures and pixel buffers
        let mut debug_texture = vec![];
        let mut debug_pixels = vec![];
        let tiles_per_row = (TILE_COUNT as f64).sqrt().ceil() as usize;
        let tiles_w = tiles_per_row * TILE_SIZE as usize;
        for bank in &tato.banks {
            let max_row = (bank.tile_count() / tiles_per_row) + 1;
            let tiles_h = max_row * TILE_SIZE as usize;
            let tiles_h = tiles_h.max(8);
            let debug_image = Image::gen_image_color(tiles_w as i32, tiles_h as i32, Color::BLACK);
            debug_texture.push(ray.load_texture_from_image(&thread, &debug_image).unwrap());
            debug_pixels.push(vec![0u8; tiles_w * tiles_h * 4]);
        }

        // Build struct
        Self {
            bg_color: Color::new(32, 32, 32, 255),
            ray,
            display_debug: false,
            display_debug_scale: 1,
            thread,
            pixels,
            debug_pixels,
            render_texture,
            debug_texture,
            font,
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

    pub fn render(&mut self, t: &mut Tato, bg_banks: &[&dyn DynamicBGMap]) {
        let mouse_x = self.ray.get_mouse_x();
        let mouse_y = self.ray.get_mouse_y();

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

        // ---------------------- Copy from framebuffer to raylib texture ----------------------

        let time = std::time::Instant::now();
        for (color, coords) in t.iter_pixels(bg_banks) {
            let i = ((coords.y as usize * t.video.width() as usize) + coords.x as usize) * 4;
            self.pixels[i] = color.r;
            self.pixels[i + 1] = color.g;
            self.pixels[i + 2] = color.b;
            self.pixels[i + 3] = color.a;
        }
        t.update_time_acc.push(time.elapsed().as_secs_f64() * 1000.0);
        println!("iter time: {:.2} ms", t.update_time_acc.average());

        self.render_texture.update_texture(&self.pixels).unwrap();

        let tiles_per_row = (TILE_COUNT as f64).sqrt().ceil() as usize;
        let tiles_w = (tiles_per_row * TILE_SIZE as usize) as i32;

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
        canvas.clear_background(self.bg_color);
        canvas.draw_texture_ex(
            &self.render_texture,
            Vector2::new(draw_rect_x as f32, draw_rect_y as f32),
            0.0,
            scale as f32,
            Color::WHITE,
        );

        // ---------------------- Copy tile pixels to debug texture ----------------------

        if self.display_debug {
            let font_size = 10 * self.display_debug_scale;
            let dark_bg = Color::new(32, 32, 32, 255);
            let light_bg = Color::new(48, 48, 48, 255);
            let rect_bg = Rect {
                x: screen_width - (tiles_w as i32 * self.display_debug_scale) - 8,
                y: font_size,
                w: tiles_w * self.display_debug_scale,
                h: screen_height - font_size - font_size,
            };
            canvas.draw_rectangle(rect_bg.x, rect_bg.y, rect_bg.w, rect_bg.h, light_bg);
            let mut layout = Frame::new(rect_bg);

            // Reset on every loop since I may change it along the way!
            layout.set_gap(1);
            layout.set_margin(1);
            layout.set_scale(self.display_debug_scale as f32);
            layout.fitting = Fitting::Clamp;
            let gap = self.display_debug_scale;
            let mut mouse_over_text = String::new();

            // Process each video memory bank
            for bank_index in 0..TILE_BANK_COUNT {
                let bank = &t.banks[bank_index];
                if bank.tile_count() == 0
                    && bank.color_count() == 0
                    && bank.sub_palette_count() == 0
                {
                    continue;
                }

                // Label
                let h = font_size / self.display_debug_scale;
                layout.push_edge(Edge::Top, h, |frame| {
                    let rect = frame.rect();
                    canvas.draw_text_ex(
                        &self.font,
                        &format!("bank {}:", bank_index),
                        Vector2::new((rect.x + gap) as f32, rect.y as f32),
                        font_size as f32,
                        1.0,
                        Color::WHITE,
                    );
                });

                layout.push_edge(Edge::Top, h, |frame| {
                    let rect = frame.rect();
                    canvas.draw_text_ex(
                        &self.font,
                        &format!(
                            "{} tiles, {} custom colors, {} sub-palettes",
                            bank.tile_count(),
                            bank.color_count(),
                            bank.sub_palette_count()
                        ),
                        Vector2::new((rect.x + gap) as f32, rect.y as f32),
                        font_size as f32 * 0.75,
                        1.0,
                        Color::WHITE,
                    );
                });

                if bank.tile_count() == 0 {
                    continue;
                }

                // Color swatches
                layout.push_edge(Edge::Top, 8, |frame| {
                    // draw bg
                    let rect = frame.rect();
                    canvas.draw_rectangle(rect.x, rect.y, rect.w, rect.h, dark_bg);
                    let swatch_w = frame.divide_width(COLORS_PER_PALETTE as u32);
                    for c in 0..COLORS_PER_PALETTE as usize {
                        let color = bank.palette[c];
                        frame.push_edge(Edge::Left, swatch_w, |swatch| {
                            let rect = swatch.rect();
                            canvas.draw_rectangle(rect.x, rect.y, rect.w, rect.h, rl_color(color));
                            // mouse over
                            if rect.contains(mouse_x, mouse_y) {
                                mouse_over_text = format!(
                                    "Color {} = {}, {}, {}, {}",
                                    c,
                                    color.r(),
                                    color.g(),
                                    color.b(),
                                    color.a()
                                );
                            }
                        });
                    }
                });

                // Subpalettes
                let columns = 4;
                let rows = (bank.sub_palette_count() as f32 / columns as f32).ceil() as u32;
                let frame_h = (rows as i32 * 4) + 2;

                layout.push_edge(Edge::Top, frame_h, |frame| {
                    let column_w = frame.divide_width(columns);
                    for column in 0..columns {
                        frame.push_edge(Edge::Left, column_w, |frame_column| {
                            frame_column.set_gap(0);
                            frame_column.set_margin(0);
                            // draw bg
                            let rect = frame_column.rect();
                            canvas.draw_rectangle(rect.x, rect.y, rect.w, rect.h, dark_bg);
                            // draw each row
                            let row_h = frame_column.divide_height(rows);
                            for row in 0..rows {
                                frame_column.push_edge(Edge::Top, row_h, |frame_row| {
                                    frame_row.set_gap(0);
                                    frame_row.set_margin(1);
                                    let subp_index =
                                        ((row * COLORS_PER_TILE as u32) + column) as usize;
                                    let subp = &bank.sub_palettes[subp_index];
                                    // draw each swatch, but only if subpalette is defined
                                    let current_item = (row * columns) + column;
                                    if current_item < bank.sub_palette_count() as u32 {
                                        let swatch_w =
                                            frame_row.divide_width(COLORS_PER_TILE as u32);
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
                                    }
                                    //mouse over
                                    if frame_row.rect().contains(mouse_x, mouse_y) {
                                        let subp_text = format!(
                                            "[{}]",
                                            subp.iter()
                                                .map(|color_id| color_id.0.to_string())
                                                .collect::<Vec<String>>()
                                                .join(",")
                                        );
                                        mouse_over_text = format!(
                                            "Sub Palette {} = Indices {}",
                                            subp_index, subp_text
                                        )
                                    }
                                });
                            }
                        });
                    }
                });

                // Tiles
                let max_row = (bank.tile_count() / tiles_per_row) + 1;
                let max_index = max_row * tiles_per_row;
                let tiles_height = max_row as i32 * TILE_SIZE as i32;
                for tile_index in 0..max_index {
                    let tile_x = tile_index % tiles_per_row;
                    let tile_y = tile_index / tiles_per_row;

                    for y in 0..TILE_SIZE as usize {
                        for x in 0..TILE_SIZE as usize {
                            let color_index = bank.tiles[tile_index].get_pixel(x as u8, y as u8);
                            let gray_value = color_index * 63; // Map 0-4 to 0-252

                            let pixel_x = tile_x * TILE_SIZE as usize + x;
                            let pixel_y = tile_y * TILE_SIZE as usize + y;
                            let pixel_offset = (pixel_y * tiles_w as usize + pixel_x) * 4;

                            // Set RGBA values
                            self.debug_pixels[bank_index][pixel_offset] = gray_value; // R
                            self.debug_pixels[bank_index][pixel_offset + 1] = gray_value; // G
                            self.debug_pixels[bank_index][pixel_offset + 2] = gray_value; // B
                            self.debug_pixels[bank_index][pixel_offset + 3] = 255; // A
                        }
                    }
                }

                layout.push_edge(Edge::Top, tiles_height, |frame_tiles| {
                    self.debug_texture[bank_index]
                        .update_texture(&self.debug_pixels[bank_index])
                        .unwrap();
                    let r = frame_tiles.rect();
                    canvas.draw_texture_ex(
                        &self.debug_texture[bank_index],
                        Vector2::new((r.x - 1) as f32, r.y as f32),
                        0.0,
                        self.display_debug_scale as f32,
                        Color::WHITE,
                    );
                    // mouse over
                    if r.contains(mouse_x, mouse_y) {
                        let col = ((mouse_x - r.x) / TILE_SIZE as i32) / self.display_debug_scale;
                        let row = ((mouse_y - r.y) / TILE_SIZE as i32) / self.display_debug_scale;
                        let tile_index = (row * tiles_per_row as i32) + col;
                        if tile_index < bank.tile_count() as i32 {
                            mouse_over_text = format!("Tile {}", tile_index);
                        }
                    }
                });
            }

            // Mouse over
            if !mouse_over_text.is_empty() {
                let size = self.font.measure_text(&mouse_over_text, font_size as f32, 1.0);
                let text_x = mouse_x - size.x as i32 - 12;
                let text_y = mouse_y + 12;
                let pad = self.display_debug_scale;
                canvas.draw_rectangle(
                    text_x - pad,
                    text_y,
                    size.x as i32 + pad + pad,
                    font_size,
                    Color::BLACK,
                );
                canvas.draw_text_ex(
                    &self.font,
                    &mouse_over_text,
                    Vector2::new(text_x as f32, text_y as f32),
                    font_size as f32,
                    1.0,
                    Color::WHITE,
                );
            }
        }
    }
}

fn rl_color(color: RGBA12) -> Color {
    Color::new(
        ((color.r() as u16 * 255) / 7) as u8,
        ((color.g() as u16 * 255) / 7) as u8,
        ((color.b() as u16 * 255) / 7) as u8,
        ((color.a() as u16 * 255) / 7) as u8,
    )
}
