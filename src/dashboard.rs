//! Generates the "Dashboard" UI, working in tandem with a Backend.
//! Provides a buffer of DrawOps that the Backend can render

use std::vec;

use crate::Tato;
use crate::backend::TextureId;
use tato_layout::{Edge, Fitting, Frame};
use tato_math::{Rect, Vec2};
use tato_video::{
    COLORS_PER_PALETTE, COLORS_PER_TILE, RGBA32, TILE_BANK_COUNT, TILE_COUNT, TILE_SIZE,
};

/// A drawing command that can be executed by any backend
#[derive(Clone, Debug)]
pub enum DrawOp {
    Rect { x: i16, y: i16, w: i16, h: i16, color: RGBA32 },
    Line { x1: i16, y1: i16, x2: i16, y2: i16, color: RGBA32 },
    Texture { id: TextureId, x: i16, y: i16, scale: f32, tint: RGBA32 },
    Text { text: String, x: f32, y: f32, size: f32, color: RGBA32 },
}

/// Backend-agnostic debug UI system that generates drawing ops
#[derive(Debug)]
pub struct Dashboard {
    pub gui_scale: i32,
    pub canvas_scale: f32,
    pub canvas_offset: Vec2<i16>,
    pub ops: Vec<DrawOp>,
    pub mouse_over_text: String,
    pub tile_pixels: [Vec<u8>; TILE_BANK_COUNT], // one vec per bank
}

impl Dashboard {
    pub fn new() -> Self {
        Self {
            gui_scale: 1,
            canvas_scale: 1.0,
            canvas_offset: Vec2 { x: 0, y: 0 },
            ops: Vec::new(),
            mouse_over_text: String::new(),
            tile_pixels: core::array::from_fn(|_| Vec::new()),
        }
    }

    /// Generate debug UI ops - completely backend agnostic
    pub fn render(&mut self, screen_size: Vec2<i16>, mouse: Vec2<i16>, tato: &Tato) {
        self.ops.clear();
        self.mouse_over_text.clear();

        // Generate ops for performance dashboard
        self.generate_text_dashboard(tato);

        // Generate ops for video memory debug
        self.generate_video_memory_debug(screen_size, mouse, tato);

        // Generate ops for debug polygons
        self.generate_debug_polygons(tato);

        // Generate tooltip command
        if !self.mouse_over_text.is_empty() {
            let width = 100;
            self.generate_tooltip(&self.mouse_over_text.clone(), width, mouse);
        }
    }

    /// Generate performance dashboard ops
    fn generate_text_dashboard(&mut self, tato: &Tato) {
        let font_size = 12.0 * self.gui_scale as f32;
        let line_height = font_size;
        let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };

        // Dashboard text from tato
        let mut y = 10.0;
        for line in tato.iter_dash_text() {
            self.ops.push(DrawOp::Text {
                text: line.to_string(),
                x: 10.0,
                y,
                size: font_size,
                color: white,
            });
            y += line_height;
        }
    }

    /// Generate tile pixel data and update texture
    fn update_tile_texture(
        &mut self,
        bank_index: usize,
        bank: &tato_video::VideoMemory<{ TILE_COUNT }>,
        tiles_per_row: usize,
    ) {
        // Early return for empty banks
        if bank.tile_count() == 0 {
            self.tile_pixels[bank_index].clear();
            return;
        }

        // Calculate actual dimensions based on tile layout
        let tile_count = bank.tile_count();
        let num_rows = (tile_count + tiles_per_row - 1) / tiles_per_row; // Ceiling division

        let tiles_w = tiles_per_row * TILE_SIZE as usize;
        let tiles_h = num_rows * TILE_SIZE as usize;

        // Allocate buffer with correct size
        let expected_size = tiles_w * tiles_h * 4; // RGBA
        self.tile_pixels[bank_index] = vec![0u8; expected_size];

        // Generate tile pixels
        for tile_index in 0..tile_count {
            let tile_x = tile_index % tiles_per_row;
            let tile_y = tile_index / tiles_per_row;

            for y in 0..TILE_SIZE as usize {
                for x in 0..TILE_SIZE as usize {
                    let color_index = bank.tiles[tile_index].get_pixel(x as u8, y as u8);
                    let gray_value = color_index * 63; // Map 0-4 to 0-252

                    let pixel_x = tile_x * TILE_SIZE as usize + x;
                    let pixel_y = tile_y * TILE_SIZE as usize + y;
                    let i = ((pixel_y * tiles_w) + pixel_x) * 4;

                    self.tile_pixels[bank_index][i] = gray_value; // R
                    self.tile_pixels[bank_index][i + 1] = gray_value; // G
                    self.tile_pixels[bank_index][i + 2] = gray_value; // B
                    self.tile_pixels[bank_index][i + 3] = 255; // A
                }
            }
        }
    }

    /// Generate video memory debug visualization ops
    fn generate_video_memory_debug(
        &mut self,
        screen_size: Vec2<i16>,
        mouse: Vec2<i16>,
        tato: &Tato,
    ) {
        let font_size = (12 * self.gui_scale) as i16;
        let dark_bg = RGBA32 { r: 32, g: 32, b: 32, a: 255 };
        let light_bg = RGBA32 { r: 48, g: 48, b: 48, a: 255 };
        let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };

        let tiles_per_row = (TILE_COUNT as f64).sqrt().ceil() as usize;
        let tiles_w = (tiles_per_row * TILE_SIZE as usize) as i16;

        // Debug panel background
        let rect_bg = Rect::new(
            screen_size.x - (tiles_w * self.gui_scale as i16) - 8,
            font_size,
            tiles_w * self.gui_scale as i16,
            screen_size.y - font_size - font_size,
        );

        self.ops.push(DrawOp::Rect {
            x: rect_bg.x,
            y: rect_bg.y,
            w: rect_bg.w,
            h: rect_bg.h,
            color: light_bg,
        });

        let mut layout = Frame::<i16>::new(rect_bg);
        layout.set_gap(1);
        layout.set_margin(1);
        layout.set_scale(self.gui_scale as f32);
        layout.fitting = Fitting::Clamp;
        let gap = self.gui_scale as i16;

        // Process each video memory bank
        for bank_index in 0..TILE_BANK_COUNT {
            let bank = &tato.banks[bank_index];
            if bank.tile_count() == 0 && bank.color_count() == 0 && bank.sub_palette_count() == 0 {
                continue;
            }

            // Bank label
            let h = font_size / self.gui_scale as i16;
            layout.push_edge(Edge::Top, h, |frame| {
                let rect = frame.rect();
                self.ops.push(DrawOp::Text {
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
                self.ops.push(DrawOp::Text {
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
            self.generate_palette_swatches(&mut layout, bank, mouse, dark_bg);

            // Sub-palette swatches
            self.generate_sub_palette_swatches(&mut layout, bank, mouse, dark_bg);

            // Tile visualization placeholder
            self.update_tile_texture(bank_index, bank, tiles_per_row);
            self.generate_tile_visualization(&mut layout, bank_index, bank, mouse, tiles_per_row);
        }
    }

    /// Generate palette swatch ops
    fn generate_palette_swatches(
        &mut self,
        layout: &mut Frame<i16>,
        bank: &tato_video::VideoMemory<{ TILE_COUNT }>,
        mouse: Vec2<i16>,
        dark_bg: RGBA32,
    ) {
        layout.push_edge(Edge::Top, 8, |frame| {
            let rect = frame.rect();
            self.ops.push(DrawOp::Rect {
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

                    self.ops.push(DrawOp::Rect {
                        x: rect.x as i16,
                        y: rect.y as i16,
                        w: rect.w as i16,
                        h: rect.h as i16,
                        color: rgba32,
                    });

                    // Mouse hover detection
                    if rect.contains(mouse.x, mouse.y) {
                        self.mouse_over_text = format!(
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
    }

    /// Generate sub-palette swatch ops
    fn generate_sub_palette_swatches(
        &mut self,
        layout: &mut Frame<i16>,
        bank: &tato_video::VideoMemory<{ TILE_COUNT }>,
        mouse: Vec2<i16>,
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

                    self.ops.push(DrawOp::Rect {
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

                            if current_item < bank.sub_palette_count() as u32
                                && subp_index < bank.sub_palettes.len()
                            {
                                let subp = &bank.sub_palettes[subp_index];
                                let swatch_w = frame_row.divide_width(COLORS_PER_TILE as u32);

                                for n in 0..COLORS_PER_TILE as usize {
                                    frame_row.push_edge(Edge::Left, swatch_w, |swatch| {
                                        let r = swatch.rect();
                                        let color_index = subp[n].0 as usize;
                                        if color_index < bank.palette.len() {
                                            let color = RGBA32::from(bank.palette[color_index]);
                                            self.ops.push(DrawOp::Rect {
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
                                if frame_row.rect().contains(mouse.x as i16, mouse.y as i16) {
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

    /// Generate tile visualization ops
    fn generate_tile_visualization(
        &mut self,
        layout: &mut Frame<i16>,
        bank_index: usize,
        bank: &tato_video::VideoMemory<{ TILE_COUNT }>,
        mouse: Vec2<i16>,
        tiles_per_row: usize,
    ) {
        let max_row = (bank.tile_count() / tiles_per_row) + 1;
        let tiles_height = max_row as i16 * TILE_SIZE as i16;

        layout.push_edge(Edge::Top, tiles_height, |frame_tiles| {
            let r = frame_tiles.rect();
            let dark_gray = RGBA32 { r: 64, g: 64, b: 64, a: 255 };

            self.ops.push(DrawOp::Rect {
                x: r.x as i16,
                y: r.y as i16,
                w: r.w as i16,
                h: r.h as i16,
                color: dark_gray,
            });

            self.ops.push(DrawOp::Texture {
                x: r.x as i16,
                y: r.y as i16,
                id: bank_index,
                scale: frame_tiles.get_scale(),
                tint: RGBA32::WHITE,
            });

            // Mouse hover detection for tiles
            if r.contains(mouse.x, mouse.y) {
                let col = ((mouse.x - r.x) / TILE_SIZE as i16) / self.gui_scale as i16;
                let row = ((mouse.y - r.y) / TILE_SIZE as i16) / self.gui_scale as i16;
                let tile_index = (row * tiles_per_row as i16) + col;
                if tile_index < bank.tile_count() as i16 {
                    self.mouse_over_text = format!("Tile {}", tile_index);
                }
            }
        });
    }

    /// Generate debug polygon ops
    fn generate_debug_polygons(&mut self, tato: &Tato) {
        let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };

        for poly in tato.iter_dash_polys(false) {
            if poly.len() >= 2 {
                for i in 0..(poly.len() - 1) {
                    let current = poly[i];
                    let next = poly[i + 1];
                    self.ops.push(DrawOp::Line {
                        x1: current.x,
                        y1: current.y,
                        x2: next.x,
                        y2: next.y,
                        color: white,
                    });
                }
            }
        }
        // World space polys (follow scrolling)
        for world_poly in tato.iter_dash_polys(true) {
            let scroll_x = tato.video.scroll_x as f32;
            let scroll_y = tato.video.scroll_y as f32;
            if world_poly.len() >= 2 {
                for i in 0..(world_poly.len() - 1) {
                    let current = world_poly[i];
                    let next = world_poly[i + 1];
                    self.ops.push(DrawOp::Line {
                        x1: ((current.x as f32 - scroll_x) * self.canvas_scale) as i16
                            + self.canvas_offset.x,
                        y1: ((current.y as f32 - scroll_y) * self.canvas_scale) as i16
                            + self.canvas_offset.y,
                        x2: ((next.x as f32 - scroll_x) * self.canvas_scale) as i16
                            + self.canvas_offset.x,
                        y2: ((next.y as f32 - scroll_y) * self.canvas_scale) as i16
                            + self.canvas_offset.y,
                        color: white,
                    });
                }
            }
        }
    }

    /// Generate tooltip command
    fn generate_tooltip(&mut self, text: &str, text_width: i16, mouse: Vec2<i16>) {
        let font_size = 12.0 * self.gui_scale as f32;
        let pad = self.gui_scale as i16;

        let text_x = mouse.x - text_width - 12;
        let text_y = mouse.y + 12;

        // Background
        let black = RGBA32 { r: 0, g: 0, b: 0, a: 255 };
        self.ops.push(DrawOp::Rect {
            x: text_x - pad,
            y: text_y,
            w: text_width + pad + pad,
            h: font_size as i16,
            color: black,
        });

        // Text
        let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };
        self.ops.push(DrawOp::Text {
            text: text.to_string(),
            x: text_x as f32,
            y: text_y as f32,
            size: font_size,
            color: white,
        });
    }

    // /// Add entity debug info (example of game-specific debug feature)
    // pub fn debug_entity(&mut self, name: &str, x: f32, y: f32, properties: &[(&str, String)]) {
    //     let font_size = 10.0 * self.scale as f32;
    //     let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };
    //     let yellow = RGBA32 { r: 255, g: 255, b: 0, a: 255 };
    //     let bg = RGBA32 { r: 0, g: 0, b: 0, a: 128 };

    //     // Entity name
    //     self.ops.push(DrawOp::Text {
    //         text: name.to_string(),
    //         x,
    //         y: y - 15.0,
    //         size: font_size,
    //         color: yellow,
    //     });

    //     // Properties
    //     let mut prop_y = y;
    //     for (key, value) in properties {
    //         let prop_text = format!("{}: {}", key, value);

    //         // Background for better readability
    //         self.ops.push(DrawOp::Rect {
    //             x: x as i16 - 2,
    //             y: prop_y as i16 - 2,
    //             w: (prop_text.len() as i16 * 6) + 4,
    //             h: font_size as i16 + 4,
    //             color: bg,
    //         });

    //         self.ops.push(DrawOp::Text {
    //             text: prop_text,
    //             x,
    //             y: prop_y,
    //             size: font_size,
    //             color: white,
    //         });
    //         prop_y += font_size + 2.0;
    //     }
    // }
}
