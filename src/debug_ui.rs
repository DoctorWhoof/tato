//! Backend-agnostic debug UI system using command-based rendering

use crate::backend::{Backend, TextureId};
use crate::Tato;
use tato_layout::{Frame, Edge, Fitting};
use tato_math::Rect;
use tato_video::{RGBA32, TILE_COUNT, TILE_BANK_COUNT, COLORS_PER_PALETTE, TILE_SIZE, COLORS_PER_TILE};

/// A drawing command that can be executed by any backend
#[derive(Clone, Debug)]
pub enum DrawOp {
    Rect { x: i16, y: i16, w: i16, h: i16, color: RGBA32 },
    Text { text: String, x: f32, y: f32, size: f32, color: RGBA32 },
    Line { x1: i16, y1: i16, x2: i16, y2: i16, color: RGBA32 },
    Texture { id: TextureId, x: f32, y: f32, scale: f32, tint: RGBA32 },
}

/// Backend-agnostic debug UI system that generates drawing commands
pub struct DebugRenderer {
    pub enabled: bool,
    pub scale: i32,
    commands: Vec<DrawOp>,
    mouse_over_text: String,
}

impl DebugRenderer {
    pub fn new() -> Self {
        Self {
            enabled: true,
            scale: 1,
            commands: Vec::new(),
            mouse_over_text: String::new(),
        }
    }

    /// Generate debug UI commands - completely backend agnostic
    pub fn render_debug_ui<B: Backend>(
        &mut self,
        backend: &B,
        tato: &Tato
    ) {
        if !self.enabled {
            return;
        }

        self.commands.clear();
        self.mouse_over_text.clear();

        let (mouse_x, mouse_y) = backend.mouse_pos();
        let (screen_width, screen_height) = backend.screen_size();

        // Generate commands for performance dashboard
        self.generate_performance_dashboard(tato);

        // Generate commands for video memory debug
        self.generate_video_memory_debug(backend, tato, mouse_x, mouse_y, screen_width, screen_height);

        // Generate commands for debug polygons
        self.generate_debug_polygons(tato);

        // Generate tooltip command
        if !self.mouse_over_text.is_empty() {
            self.generate_tooltip(backend, &self.mouse_over_text.clone(), mouse_x, mouse_y);
        }
    }

    /// Execute all collected commands using the Backend trait
    pub fn execute_commands<B: Backend>(&self, backend: &mut B) {
        for cmd in &self.commands {
            match cmd {
                DrawOp::Rect { x, y, w, h, color } => {
                    backend.draw_rect(*x, *y, *w, *h, *color);
                }
                DrawOp::Text { text, x, y, size, color } => {
                    backend.draw_text(text, *x, *y, *size, *color);
                }
                DrawOp::Line { x1, y1, x2, y2, color } => {
                    backend.draw_line(*x1, *y1, *x2, *y2, *color);
                }
                DrawOp::Texture { id, x, y, scale, tint } => {
                    backend.draw_texture(*id, *x, *y, *scale, *tint);
                }
            }
        }
    }

    /// Get the list of drawing commands for custom execution
    pub fn get_commands(&self) -> &[DrawOp] {
        &self.commands
    }

    /// Add a custom drawing command (for game-specific debug features)
    pub fn add_command(&mut self, command: DrawOp) {
        self.commands.push(command);
    }

    /// Generate performance dashboard commands
    fn generate_performance_dashboard(&mut self, tato: &Tato) {
        let font_size = 12.0 * self.scale as f32;
        let line_height = font_size;
        let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };

        let mut y = 10.0;

        // Dashboard text from tato
        for line in tato.get_dash_text() {
            self.commands.push(DrawOp::Text {
                text: line.to_string(),
                x: 10.0,
                y,
                size: font_size,
                color: white,
            });
            y += line_height;
        }

        // Performance metrics
        self.commands.push(DrawOp::Text {
            text: format!("fps: {:.2}", 1.0 / tato.elapsed_time()),
            x: 10.0,
            y,
            size: font_size,
            color: white,
        });
        y += line_height;

        self.commands.push(DrawOp::Text {
            text: format!("elapsed: {:.2} ms", tato.elapsed_time() * 1000.0),
            x: 10.0,
            y,
            size: font_size,
            color: white,
        });
        y += line_height;

        // Additional performance metrics that were in the backend
        self.commands.push(DrawOp::Text {
            text: "Debug UI: Backend-agnostic".to_string(),
            x: 10.0,
            y,
            size: font_size * 0.8,
            color: RGBA32 { r: 200, g: 200, b: 200, a: 255 },
        });
    }

    /// Generate video memory debug visualization commands
    fn generate_video_memory_debug<B: Backend>(
        &mut self,
        _backend: &B,
        tato: &Tato,
        mouse_x: i16,
        mouse_y: i16,
        screen_width: i16,
        screen_height: i16,
    ) {
        let font_size = (12 * self.scale) as i16;
        let dark_bg = RGBA32 { r: 32, g: 32, b: 32, a: 255 };
        let light_bg = RGBA32 { r: 48, g: 48, b: 48, a: 255 };
        let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };

        let tiles_per_row = (TILE_COUNT as f64).sqrt().ceil() as usize;
        let tiles_w = (tiles_per_row * TILE_SIZE as usize) as i16;

        // Debug panel background
        let rect_bg = Rect::new(
            screen_width - (tiles_w * self.scale as i16) - 8,
            font_size,
            tiles_w * self.scale as i16,
            screen_height - font_size - font_size,
        );

        self.commands.push(DrawOp::Rect {
            x: rect_bg.x,
            y: rect_bg.y,
            w: rect_bg.w,
            h: rect_bg.h,
            color: light_bg,
        });

        let mut layout = Frame::<i16>::new(rect_bg);
        layout.set_gap(1);
        layout.set_margin(1);
        layout.set_scale(self.scale as f32);
        layout.fitting = Fitting::Clamp;
        let gap = self.scale as i16;

        // Process each video memory bank
        for bank_index in 0..TILE_BANK_COUNT {
            let bank = &tato.banks[bank_index];
            if bank.tile_count() == 0 && bank.color_count() == 0 && bank.sub_palette_count() == 0 {
                continue;
            }

            // Bank label
            let h = font_size / self.scale as i16;
            layout.push_edge(Edge::Top, h, |frame| {
                let rect = frame.rect();
                self.commands.push(DrawOp::Text {
                    text: format!("bank {}:", bank_index),
                    x: (rect.x + gap) as f32,
                    y: rect.y as f32,
                    size: font_size as f32,
                    color: white,
                });
            });

            // Bank info
            layout.push_edge(Edge::Top, h, |frame| {
                let rect = frame.rect();
                self.commands.push(DrawOp::Text {
                    text: format!(
                        "{} tiles, {} custom colors, {} sub-palettes",
                        bank.tile_count(),
                        bank.color_count(),
                        bank.sub_palette_count()
                    ),
                    x: (rect.x + gap) as f32,
                    y: rect.y as f32,
                    size: font_size as f32 * 0.75,
                    color: white,
                });
            });

            if bank.tile_count() == 0 {
                continue;
            }

            // Color palette swatches
            self.generate_palette_swatches(&mut layout, bank, mouse_x, mouse_y, dark_bg);

            // Sub-palette swatches
            self.generate_sub_palette_swatches(&mut layout, bank, mouse_x, mouse_y, dark_bg);

            // Tile visualization placeholder
            self.generate_tile_visualization(&mut layout, bank, mouse_x, mouse_y, tiles_per_row);
        }
    }

    /// Generate palette swatch commands
    fn generate_palette_swatches(
        &mut self,
        layout: &mut Frame<i16>,
        bank: &tato_video::VideoMemory<{TILE_COUNT}>,
        mouse_x: i16,
        mouse_y: i16,
        dark_bg: RGBA32,
    ) {
        layout.push_edge(Edge::Top, 8, |frame| {
            let rect = frame.rect();
            self.commands.push(DrawOp::Rect {
                x: rect.x as i16,
                y: rect.y as i16,
                w: rect.w as i16,
                h: rect.h as i16,
                color: dark_bg,
            });

            let swatch_w = frame.divide_width(COLORS_PER_PALETTE as u32);
            for c in 0..COLORS_PER_PALETTE as usize {
                frame.push_edge(Edge::Left, swatch_w, |swatch| {
                    let rect = swatch.rect();
                    let color = bank.palette[c];
                    let rgba32 = RGBA32::from(color);

                    self.commands.push(DrawOp::Rect {
                        x: rect.x as i16,
                        y: rect.y as i16,
                        w: rect.w as i16,
                        h: rect.h as i16,
                        color: rgba32,
                    });

                    // Mouse hover detection
                    if rect.contains(mouse_x, mouse_y) {
                        self.mouse_over_text = format!(
                            "Color {} = {}, {}, {}, {}",
                            c, color.r(), color.g(), color.b(), color.a()
                        );
                    }
                });
            }
        });
    }

    /// Generate sub-palette swatch commands
    fn generate_sub_palette_swatches(
        &mut self,
        layout: &mut Frame<i16>,
        bank: &tato_video::VideoMemory<{TILE_COUNT}>,
        mouse_x: i16,
        mouse_y: i16,
        dark_bg: RGBA32,
    ) {
        let columns = 6;
        let rows = (bank.sub_palette_count() as f32 / columns as f32).ceil() as u32;
        let frame_h = (rows as i16 * 4) + 2;

        layout.push_edge(Edge::Top, frame_h, |frame| {
            let column_w = frame.divide_width(columns);
            for column in 0..columns {
                frame.push_edge(Edge::Left, column_w, |frame_column| {
                    frame_column.set_gap(0);
                    frame_column.set_margin(0);
                    let rect = frame_column.rect();

                    self.commands.push(DrawOp::Rect {
                        x: rect.x as i16,
                        y: rect.y as i16,
                        w: rect.w as i16,
                        h: rect.h as i16,
                        color: dark_bg,
                    });

                    let row_h = frame_column.divide_height(rows);
                    for row in 0..rows {
                        frame_column.push_edge(Edge::Top, row_h, |frame_row| {
                            frame_row.set_gap(0);
                            frame_row.set_margin(1);
                            let subp_index = ((row * COLORS_PER_TILE as u32) + column) as usize;
                            let current_item = (row * columns) + column;

                            if current_item < bank.sub_palette_count() as u32 && subp_index < bank.sub_palettes.len() {
                                let subp = &bank.sub_palettes[subp_index];
                                let swatch_w = frame_row.divide_width(COLORS_PER_TILE as u32);

                                for n in 0..COLORS_PER_TILE as usize {
                                    frame_row.push_edge(Edge::Left, swatch_w, |swatch| {
                                        let r = swatch.rect();
                                        let color_index = subp[n].0 as usize;
                                        if color_index < bank.palette.len() {
                                            let color = RGBA32::from(bank.palette[color_index]);
                                            self.commands.push(DrawOp::Rect {
                                                x: r.x as i16,
                                                y: r.y as i16,
                                                w: r.w as i16,
                                                h: r.h as i16,
                                                color,
                                            });
                                        }
                                    });
                                }

                                // Mouse hover detection
                                if frame_row.rect().contains(mouse_x as i16, mouse_y as i16) {
                                    let subp_text = format!(
                                        "[{}]",
                                        subp.iter()
                                            .map(|color_id| color_id.0.to_string())
                                            .collect::<Vec<String>>()
                                            .join(",")
                                    );
                                    self.mouse_over_text = format!(
                                        "Sub Palette {} = Indices {}",
                                        subp_index, subp_text
                                    );
                                }
                            }
                        });
                    }
                });
            }
        });
    }

    /// Generate tile visualization commands
    fn generate_tile_visualization(
        &mut self,
        layout: &mut Frame<i16>,
        bank: &tato_video::VideoMemory<{TILE_COUNT}>,
        mouse_x: i16,
        mouse_y: i16,
        tiles_per_row: usize,
    ) {
        let max_row = (bank.tile_count() / tiles_per_row) + 1;
        let tiles_height = max_row as i16 * TILE_SIZE as i16;

        layout.push_edge(Edge::Top, tiles_height, |frame_tiles| {
            let r = frame_tiles.rect();
            let dark_gray = RGBA32 { r: 64, g: 64, b: 64, a: 255 };

            self.commands.push(DrawOp::Rect {
                x: r.x as i16,
                y: r.y as i16,
                w: r.w as i16,
                h: r.h as i16,
                color: dark_gray,
            });

            // Mouse hover detection for tiles
            if r.contains(mouse_x, mouse_y) {
                let col = ((mouse_x - r.x) / TILE_SIZE as i16) / self.scale as i16;
                let row = ((mouse_y - r.y) / TILE_SIZE as i16) / self.scale as i16;
                let tile_index = (row * tiles_per_row as i16) + col;
                if tile_index < bank.tile_count() as i16 {
                    self.mouse_over_text = format!("Tile {}", tile_index);
                }
            }
        });
    }

    /// Generate debug polygon commands
    fn generate_debug_polygons(&mut self, tato: &Tato) {
        let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };

        for poly in tato.get_dash_polys() {
            if poly.len() >= 2 {
                for i in 0..(poly.len() - 1) {
                    let current = poly[i];
                    let next = poly[i + 1];
                    self.commands.push(DrawOp::Line {
                        x1: current.x,
                        y1: current.y,
                        x2: next.x,
                        y2: next.y,
                        color: white,
                    });
                }
            }
        }
    }

    /// Generate tooltip command
    fn generate_tooltip<B: Backend>(&mut self, backend: &B, text: &str, mouse_x: i16, mouse_y: i16) {
        let font_size = 12.0 * self.scale as f32;
        let (text_w, _text_h) = backend.measure_text(text, font_size);
        let pad = self.scale as i16;

        let text_x = mouse_x - text_w as i16 - 12;
        let text_y = mouse_y + 12;

        // Background
        let black = RGBA32 { r: 0, g: 0, b: 0, a: 255 };
        self.commands.push(DrawOp::Rect {
            x: text_x - pad,
            y: text_y,
            w: text_w as i16 + pad + pad,
            h: font_size as i16,
            color: black,
        });

        // Text
        let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };
        self.commands.push(DrawOp::Text {
            text: text.to_string(),
            x: text_x as f32,
            y: text_y as f32,
            size: font_size,
            color: white,
        });
    }

    /// Toggle debug mode
    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }

    /// Set debug scale
    pub fn set_scale(&mut self, scale: i32) {
        self.scale = scale.max(1);
    }

    /// Handle debug input (call this in your game loop)
    pub fn handle_debug_input<B: Backend>(&mut self, _backend: &B) {
        // Note: This requires backend to expose key state - you may need to implement this
        // For now, games should handle debug input themselves and call toggle()/set_scale()
    }

    /// Convenience method for easy migration from old debug system
    /// Call this instead of the old backend.render() debug functionality
    pub fn render_and_execute<B: Backend>(&mut self, backend: &mut B, tato: &Tato) {
        if self.enabled {
            self.render_debug_ui(backend, tato);
            self.execute_commands(backend);
        }
    }

    /// Handle standard debug input keys (TAB, +, -)
    /// Returns true if any debug input was handled
    pub fn handle_standard_input(&mut self, tab_pressed: bool, plus_pressed: bool, minus_pressed: bool) -> bool {
        let mut handled = false;

        if tab_pressed {
            self.toggle();
            handled = true;
        }
        if plus_pressed {
            self.set_scale(self.scale + 1);
            handled = true;
        }
        if minus_pressed && self.scale > 1 {
            self.set_scale(self.scale - 1);
            handled = true;
        }

        handled
    }

    // === Game-Specific Debug Extensions ===

    /// Add entity debug info (example of game-specific debug feature)
    pub fn debug_entity(&mut self, name: &str, x: f32, y: f32, properties: &[(&str, String)]) {
        if !self.enabled {
            return;
        }

        let font_size = 10.0 * self.scale as f32;
        let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };
        let yellow = RGBA32 { r: 255, g: 255, b: 0, a: 255 };
        let bg = RGBA32 { r: 0, g: 0, b: 0, a: 128 };

        // Entity name
        self.commands.push(DrawOp::Text {
            text: name.to_string(),
            x,
            y: y - 15.0,
            size: font_size,
            color: yellow,
        });

        // Properties
        let mut prop_y = y;
        for (key, value) in properties {
            let prop_text = format!("{}: {}", key, value);

            // Background for better readability
            self.commands.push(DrawOp::Rect {
                x: x as i16 - 2,
                y: prop_y as i16 - 2,
                w: (prop_text.len() as i16 * 6) + 4,
                h: font_size as i16 + 4,
                color: bg,
            });

            self.commands.push(DrawOp::Text {
                text: prop_text,
                x,
                y: prop_y,
                size: font_size,
                color: white,
            });
            prop_y += font_size + 2.0;
        }
    }

    /// Add collision box debug visualization
    pub fn debug_collision_box(&mut self, rect: Rect<i16>, color: Option<RGBA32>) {
        if !self.enabled {
            return;
        }

        let debug_color = color.unwrap_or(RGBA32 { r: 255, g: 0, b: 0, a: 128 });

        // Draw collision box outline
        self.commands.push(DrawOp::Line {
            x1: rect.x,
            y1: rect.y,
            x2: rect.x + rect.w,
            y2: rect.y,
            color: debug_color,
        });
        self.commands.push(DrawOp::Line {
            x1: rect.x + rect.w,
            y1: rect.y,
            x2: rect.x + rect.w,
            y2: rect.y + rect.h,
            color: debug_color,
        });
        self.commands.push(DrawOp::Line {
            x1: rect.x + rect.w,
            y1: rect.y + rect.h,
            x2: rect.x,
            y2: rect.y + rect.h,
            color: debug_color,
        });
        self.commands.push(DrawOp::Line {
            x1: rect.x,
            y1: rect.y + rect.h,
            x2: rect.x,
            y2: rect.y,
            color: debug_color,
        });
    }
}

impl Default for DebugRenderer {
    fn default() -> Self {
        Self::new()
    }
}
