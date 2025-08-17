//! Generates the "Dashboard" UI, working in tandem with a Backend.
//! Provides a buffer of DrawOps that the Backend can render

use std::usize::MAX;

use crate::prelude::{Edge, Frame, Rect, Tato};
use tato_video::{RGBA32, TILE_BANK_COUNT, TILE_COUNT, TILE_SIZE, VideoMemory};

mod ops;
pub use ops::*;

mod args;
pub use args::*;
use tato_arena::{Arena, Buffer, Text};

const ARENA_SIZE: usize = 13000;
const MAX_LINES: usize = 100;
const TEXT_CHAR_LEN: usize = 50 ;
const DRAW_OP_LEN: usize = ARENA_SIZE / size_of::<DrawOp>();

/// Backend-agnostic debug UI system that generates drawing ops
#[derive(Debug)]
pub struct Dashboard {
    pub arena_text: Arena<ARENA_SIZE, u16>,
    pub arena_ops: Arena<ARENA_SIZE, u16>,
    pub tile_pixels: [Vec<u8>; TILE_BANK_COUNT], // one vec per bank
    pub mouse_over_text: String,
    pub text_cursor_y: f32,
    pub font_size: f32,
    pub console: bool,
    pub console_buffer: Buffer<Text, u16>,
    // TODO: Change to use arena buffers
    pub ops: Buffer<DrawOp>,
    additional_text: Buffer<Text>,
}

const DARK_GRAY: RGBA32 = RGBA32 { r: 18, g: 18, b: 18, a: 200 };

impl Dashboard {
    pub const PANEL_WIDTH: i16 = 150;
    pub const MARGIN: i16 = 10;

    pub fn new() -> Self {
        let mut arena_text = Arena::new();
        let console_buffer = Buffer::text_multi_buffer(
            &mut arena_text,
            TEXT_CHAR_LEN as u16,
            MAX_LINES as u16,
            true,
        )
        .unwrap();
        let additional_text = Buffer::text_multi_buffer(
            &mut arena_text,
            TEXT_CHAR_LEN as u16,
            MAX_LINES as u16,
            true,
        )
        .unwrap();

        let mut arena_ops = Arena::new();
        let ops = Buffer::new(&mut arena_ops, DRAW_OP_LEN as u16).unwrap();
        Self {
            arena_text,
            arena_ops,
            tile_pixels: core::array::from_fn(|_| Vec::new()),
            ops,
            additional_text,
            mouse_over_text: String::default(),
            text_cursor_y: 0.0,
            font_size: 8.0,
            console: true,
            console_buffer,
        }
    }

    /// Must be called at the end of each frame, or the arena will fill up.
    pub fn clear(&mut self) {
        self.mouse_over_text.clear();

        self.arena_text.clear();
        self.console_buffer = Buffer::text_multi_buffer(
            &mut self.arena_text,
            TEXT_CHAR_LEN as u16,
            MAX_LINES as u16,
            true,
        )
        .unwrap();
        self.additional_text = Buffer::text_multi_buffer(
            &mut self.arena_text,
            TEXT_CHAR_LEN as u16,
            MAX_LINES as u16,
            true,
        )
        .unwrap();

        self.arena_ops.clear();
        self.ops = Buffer::new(&mut self.arena_ops, DRAW_OP_LEN as u16).unwrap();
        self.text_cursor_y = 0.0;
    }

    /// Returns a reference to all DrawOps
    pub fn ops(&self) -> Option<&[DrawOp]> {
        self.arena_ops.get_slice(&self.ops.slice)
    }

    pub fn add_text(&mut self, text: &str) {
        let text = Text::from_str(&mut self.arena_text, text).unwrap();
        self.additional_text.push(&mut self.arena_text, text).unwrap();
    }

    /// Generate debug UI ops
    pub fn render(&mut self, tato: &Tato, args: DashArgs) {
        let screen_rect = Rect { x: 0, y: 0, w: args.screen_size.x, h: args.screen_size.y };
        let mut layout = Frame::new(screen_rect);

        layout.set_scale(args.gui_scale);
        layout.set_margin(Self::MARGIN);
        layout.set_gap(10);

        self.add_left_panel(&mut layout, tato);
        self.add_right_panel(&mut layout, tato);
        self.generate_console_ops(&mut layout, tato);

        // // Generate ops for debug polygons
        // self.generate_debug_polygons_ops(tato, args.canvas_pos, args.canvas_scale);

        // // Generate tooltip command
        // if !self.mouse_over_text.is_empty() {
        //     let width = 100;
        //     self.generate_tooltip_ops(&self.mouse_over_text.clone(), width, args.mouse);
        // }
    }

    fn add_left_panel(&mut self, layout: &mut Frame<i16>, tato: &Tato) {
        layout.push_edge(Edge::Left, Self::PANEL_WIDTH, |panel| {
            panel.set_margin(5);
            panel.set_gap(0);
            self.ops
                .push(&mut self.arena_ops, DrawOp::Rect { rect: panel.rect(), color: DARK_GRAY })
                .unwrap();

            {
                let text_arena_size = self.arena_text.used();
                let op_arena_size = self.arena_ops.used();
                let mut push_text = |text: &str| {
                    let text = Text::from_str(&mut self.arena_text, text).unwrap();
                    self.additional_text.push(&mut self.arena_text, text).unwrap();
                };
                push_text(&format!("Debug arena size: {} Kb", tato.debug_arena.used() / 1024));
                push_text(&format!("Asset arena size: {} Kb", tato.assets.arena.used() / 1024));
                push_text(&format!("Dash Text arena: {} Kb", text_arena_size / 1024));
                push_text(&format!("Dash Op arena: {} Kb", op_arena_size / 1024));
                push_text(&format!("fps: {:.2}", 1.0 / tato.elapsed_time()));
                push_text(&format!("elapsed: {:.2} ms", tato.elapsed_time() * 1000.0));
                push_text("------------------------");
                for line in tato.iter_dash_text() {
                    push_text(line);
                }
            }

            // Additional debug info
            if let Some(iter) = self.additional_text.iter(&self.arena_text) {
                for text in iter {
                    panel.push_edge(Edge::Top, self.font_size as i16, |text_frame| {
                        let line_height = self.font_size * text_frame.get_scale();
                        let rect = text_frame.rect();
                        self.ops
                            .push(
                                &mut self.arena_ops,
                                DrawOp::Text {
                                    text: text.clone(),
                                    x: rect.x,
                                    y: rect.y,
                                    size: line_height,
                                    color: RGBA32::WHITE,
                                },
                            )
                            .unwrap();
                    });
                }
            }
        });
    }

    fn add_right_panel(&mut self, layout: &mut Frame<i16>, tato: &Tato) {
        layout.push_edge(Edge::Right, Self::PANEL_WIDTH, |panel| {
            panel.set_margin(5);
            panel.set_gap(0);
            self.ops
                .push(&mut self.arena_ops, DrawOp::Rect { rect: panel.rect(), color: DARK_GRAY })
                .unwrap();
        });
    }

    // pub fn add_text_op(&mut self, text: &str) {
    //     let line_height = self.font_size * self.scale;
    //     self.ops.push(DrawOp::Text {
    //         text: text.to_string(),
    //         x: 10.0,
    //         y: self.text_cursor_y,
    //         size: line_height,
    //         color: RGBA32::WHITE,
    //     });
    //     self.text_cursor_y += line_height;
    // }

    fn generate_console_ops(&mut self, layout: &mut Frame<i16>, tato: &Tato) {
        if !self.console {
            return;
        }
        layout.push_edge(Edge::Bottom, 80, |console| {
            self.ops
                .push(
                    &mut self.arena_ops,
                    DrawOp::Rect {
                        rect: console.rect(),
                        color: RGBA32 { r: 18, g: 18, b: 18, a: 230 },
                    },
                )
                .unwrap();
        });
    }

    // /// Generate performance dashboard ops
    // fn generate_text_dashboard_ops(&mut self, tato: &Tato) {
    //     // Additional debug info
    //     let arena_size = &format!("Debug arena size: {} Kb", tato.debug_arena.used() / 1024);
    //     self.add_text_op(arena_size);

    //     let asset_size = &format!("Asset arena size: {} Kb", tato.assets.arena.used() / 1024);
    //     self.add_text_op(asset_size);

    //     let fps = &format!("fps: {:.2}", 1.0 / tato.elapsed_time());
    //     self.add_text_op(fps);

    //     let elapsed = &format!("elapsed: {:.2} ms", tato.elapsed_time() * 1000.0);
    //     self.add_text_op(elapsed);

    //     self.add_text_op("------------------------");

    //     // Dashboard text from tato
    //     for line in tato.iter_dash_text() {
    //         self.add_text_op(line);
    //     }
    // }

    /// Generate tile pixel data and update texture
    fn update_tile_texture(
        &mut self,
        bank_index: usize,
        bank: &VideoMemory<{ TILE_COUNT }>,
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

        let w = tiles_per_row * TILE_SIZE as usize;
        let h = num_rows * TILE_SIZE as usize;

        let expected_size = w * h * 4; // RGBA
        if expected_size != self.tile_pixels[bank_index].len() {
            println!("Updating tile texture on bank {}", bank_index);

            // Allocate buffer with correct size
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
                        let i = ((pixel_y * w) + pixel_x) * 4;

                        self.tile_pixels[bank_index][i] = gray_value; // R
                        self.tile_pixels[bank_index][i + 1] = gray_value; // G
                        self.tile_pixels[bank_index][i + 2] = gray_value; // B
                        self.tile_pixels[bank_index][i + 3] = 255; // A
                    }
                }
            }
        }
    }

    // /// Generate video memory debug visualization ops
    // fn generate_video_memory_debug_ops(
    //     &mut self,
    //     tato: &Tato,
    //     screen_size: Vec2<i16>,
    //     mouse: Vec2<i16>,
    // ) {
    //     let font_size = (self.font_size * self.scale) as i16;
    //     let dark_bg = RGBA32 { r: 32, g: 32, b: 32, a: 255 };
    //     let light_bg = RGBA32 { r: 48, g: 48, b: 48, a: 255 };
    //     let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };

    //     let tiles_per_row = (TILE_COUNT as f64).sqrt().ceil() as usize;
    //     let tiles_w = (tiles_per_row * TILE_SIZE as usize) as i16;

    //     // Debug panel background
    //     let rect_bg = Rect::new(
    //         screen_size.x - (tiles_w * self.scale as i16) - 8,
    //         self.font_size as i16,
    //         tiles_w * self.scale as i16,
    //         screen_size.y - (self.font_size + self.font_size) as i16,
    //     );

    //     self.ops.push(DrawOp::Rect { rect: rect_bg, color: light_bg });

    //     let mut layout = Frame::<i16>::new(rect_bg);
    //     layout.set_gap(1);
    //     layout.set_margin(1);
    //     layout.set_scale(self.scale);
    //     layout.fitting = Fitting::Clamp;
    //     let gap = self.scale as i16;

    //     // Process each video memory bank
    //     for bank_index in 0..TILE_BANK_COUNT {
    //         let bank = &tato.banks[bank_index];
    //         if bank.tile_count() == 0 && bank.color_count() == 0 && bank.sub_palette_count() == 0 {
    //             continue;
    //         }

    //         // Bank label
    //         let h = font_size / self.scale as i16;
    //         layout.push_edge(Edge::Top, h, |frame| {
    //             let rect = frame.rect();
    //             self.ops.push(DrawOp::Text {
    //                 text: format!("bank {}:", bank_index),
    //                 x: rect.x + gap,
    //                 y: rect.y,
    //                 size: font_size as f32,
    //                 color: white,
    //             });
    //         });

    //         // Bank info
    //         layout.push_edge(Edge::Top, h, |frame| {
    //             let rect = frame.rect();
    //             self.ops.push(DrawOp::Text {
    //                 text: format!(
    //                     "{} tiles, {} custom colors, {} sub-palettes",
    //                     bank.tile_count(),
    //                     bank.color_count(),
    //                     bank.sub_palette_count()
    //                 ),
    //                 x: rect.x + gap,
    //                 y: rect.y,
    //                 size: font_size as f32 * 0.75,
    //                 color: white,
    //             });
    //         });

    //         if bank.tile_count() == 0 {
    //             continue;
    //         }

    //         // Color palette swatches
    //         self.generate_palette_swatches_ops(&mut layout, bank, mouse, dark_bg);

    //         // Sub-palette swatches
    //         self.generate_sub_palette_swatches_ops(&mut layout, bank, mouse, dark_bg);

    //         // Tile visualization placeholder
    //         self.update_tile_texture(bank_index, bank, tiles_per_row);
    //         self.generate_tile_visualization_ops(
    //             &mut layout,
    //             bank_index,
    //             bank,
    //             mouse,
    //             tiles_per_row,
    //         );
    //     }
    // }

    // /// Generate palette swatch ops
    // fn generate_palette_swatches_ops(
    //     &mut self,
    //     layout: &mut Frame<i16>,
    //     bank: &VideoMemory<{ TILE_COUNT }>,
    //     mouse: Vec2<i16>,
    //     dark_bg: RGBA32,
    // ) {
    //     layout.push_edge(Edge::Top, 8, |frame| {
    //         let rect = frame.rect();
    //         self.ops.push(DrawOp::Rect { rect, color: dark_bg });

    //         let swatch_w = frame.divide_width(COLORS_PER_PALETTE as u32);
    //         for c in 0..COLORS_PER_PALETTE as usize {
    //             frame.push_edge(Edge::Left, swatch_w, |swatch| {
    //                 let rect = swatch.rect();
    //                 let color = bank.palette[c];
    //                 let rgba32 = RGBA32::from(color);

    //                 self.ops.push(DrawOp::Rect { rect, color: rgba32 });

    //                 // Mouse hover detection
    //                 if rect.contains(mouse.x, mouse.y) {
    //                     self.mouse_over_text = format!(
    //                         "Color {} = {}, {}, {}, {}",
    //                         c,
    //                         color.r(),
    //                         color.g(),
    //                         color.b(),
    //                         color.a()
    //                     );
    //                 }
    //             });
    //         }
    //     });
    // }

    // /// Generate sub-palette swatch ops
    // fn generate_sub_palette_swatches_ops(
    //     &mut self,
    //     layout: &mut Frame<i16>,
    //     bank: &VideoMemory<{ TILE_COUNT }>,
    //     mouse: Vec2<i16>,
    //     dark_bg: RGBA32,
    // ) {
    //     let columns = 6;
    //     let rows = (bank.sub_palette_count() as f32 / columns as f32).ceil() as u32;
    //     let frame_h = (rows as i16 * 4) + 2;

    //     layout.push_edge(Edge::Top, frame_h, |frame| {
    //         let column_w = frame.divide_width(columns);
    //         for column in 0..columns {
    //             frame.push_edge(Edge::Left, column_w, |frame_column| {
    //                 frame_column.set_gap(0);
    //                 frame_column.set_margin(0);
    //                 let rect = frame_column.rect();

    //                 self.ops.push(DrawOp::Rect { rect, color: dark_bg });

    //                 let row_h = frame_column.divide_height(rows);
    //                 for row in 0..rows {
    //                     frame_column.push_edge(Edge::Top, row_h, |frame_row| {
    //                         frame_row.set_gap(0);
    //                         frame_row.set_margin(1);
    //                         let subp_index = ((row * COLORS_PER_TILE as u32) + column) as usize;
    //                         let current_item = (row * columns) + column;

    //                         if current_item < bank.sub_palette_count() as u32
    //                             && subp_index < bank.sub_palettes.len()
    //                         {
    //                             let subp = &bank.sub_palettes[subp_index];
    //                             let swatch_w = frame_row.divide_width(COLORS_PER_TILE as u32);

    //                             for n in 0..COLORS_PER_TILE as usize {
    //                                 frame_row.push_edge(Edge::Left, swatch_w, |swatch| {
    //                                     let r = swatch.rect();
    //                                     let color_index = subp[n].0 as usize;
    //                                     if color_index < bank.palette.len() {
    //                                         let color = RGBA32::from(bank.palette[color_index]);
    //                                         self.ops.push(DrawOp::Rect { rect: r, color });
    //                                     }
    //                                 });
    //                             }

    //                             // Mouse hover detection
    //                             if frame_row.rect().contains(mouse.x as i16, mouse.y as i16) {
    //                                 let subp_text = format!(
    //                                     "[{}]",
    //                                     subp.iter()
    //                                         .map(|color_id| color_id.0.to_string())
    //                                         .collect::<Vec<String>>()
    //                                         .join(",")
    //                                 );
    //                                 self.mouse_over_text = format!(
    //                                     "Sub Palette {} = Indices {}",
    //                                     subp_index, subp_text
    //                                 );
    //                             }
    //                         }
    //                     });
    //                 }
    //             });
    //         }
    //     });
    // }

    // /// Generate tile visualization ops
    // fn generate_tile_visualization_ops(
    //     &mut self,
    //     layout: &mut Frame<i16>,
    //     bank_index: usize,
    //     bank: &VideoMemory<{ TILE_COUNT }>,
    //     mouse: Vec2<i16>,
    //     tiles_per_row: usize,
    // ) {
    //     let max_row = (bank.tile_count() / tiles_per_row) + 1;
    //     let tiles_height = max_row as i16 * TILE_SIZE as i16;

    //     layout.push_edge(Edge::Top, tiles_height, |frame_tiles| {
    //         let rect = frame_tiles.rect();
    //         let dark_gray = RGBA32 { r: 64, g: 64, b: 64, a: 255 };

    //         self.ops.push(DrawOp::Rect { rect, color: dark_gray });

    //         self.ops.push(DrawOp::Texture {
    //             x: rect.x as i16,
    //             y: rect.y as i16,
    //             id: bank_index,
    //             scale: frame_tiles.get_scale(),
    //             tint: RGBA32::WHITE,
    //         });

    //         // Mouse hover detection for tiles
    //         if rect.contains(mouse.x, mouse.y) {
    //             let col = ((mouse.x - rect.x) / TILE_SIZE as i16) / self.scale as i16;
    //             let row = ((mouse.y - rect.y) / TILE_SIZE as i16) / self.scale as i16;
    //             let tile_index = (row * tiles_per_row as i16) + col;
    //             if tile_index < bank.tile_count() as i16 {
    //                 self.mouse_over_text = format!("Tile {}", tile_index);
    //             }
    //         }
    //     });
    // }

    // /// Generate debug polygon ops
    // fn generate_debug_polygons_ops(
    //     &mut self,
    //     tato: &Tato,
    //     canvas_pos: Vec2<i16>,
    //     canvas_scale: f32,
    // ) {
    //     let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };

    //     for poly in tato.iter_dash_polys(false) {
    //         if poly.len() >= 2 {
    //             for i in 0..(poly.len() - 1) {
    //                 let current = poly[i];
    //                 let next = poly[i + 1];
    //                 self.ops.push(DrawOp::Line {
    //                     x1: current.x,
    //                     y1: current.y,
    //                     x2: next.x,
    //                     y2: next.y,
    //                     color: white,
    //                 });
    //             }
    //         }
    //     }
    //     // World space polys (follow scrolling)
    //     for world_poly in tato.iter_dash_polys(true) {
    //         let scroll_x = tato.video.scroll_x as f32;
    //         let scroll_y = tato.video.scroll_y as f32;
    //         if world_poly.len() >= 2 {
    //             for i in 0..(world_poly.len() - 1) {
    //                 let current = world_poly[i];
    //                 let next = world_poly[i + 1];
    //                 self.ops.push(DrawOp::Line {
    //                     x1: ((current.x as f32 - scroll_x) * canvas_scale) as i16 + canvas_pos.x,
    //                     y1: ((current.y as f32 - scroll_y) * canvas_scale) as i16 + canvas_pos.y,
    //                     x2: ((next.x as f32 - scroll_x) * canvas_scale) as i16 + canvas_pos.x,
    //                     y2: ((next.y as f32 - scroll_y) * canvas_scale) as i16 + canvas_pos.y,
    //                     color: white,
    //                 });
    //             }
    //         }
    //     }
    // }

    // /// Generate tooltip command
    // fn generate_tooltip_ops(&mut self, text: &str, text_width: i16, mouse: Vec2<i16>) {
    //     let font_size = 12.0 * self.scale as f32;
    //     let pad = self.scale as i16;

    //     let text_x = mouse.x - text_width - 12;
    //     let text_y = mouse.y + 12;

    //     // Background
    //     let black = RGBA32 { r: 0, g: 0, b: 0, a: 255 };
    //     self.ops.push(DrawOp::Rect {
    //         rect: Rect {
    //             x: text_x - pad,
    //             y: text_y,
    //             w: text_width + pad + pad,
    //             h: font_size as i16,
    //         },
    //         color: black,
    //     });

    //     // Text
    //     let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };
    //     self.ops.push(DrawOp::Text {
    //         text: text.to_string(),
    //         x: text_x,
    //         y: text_y,
    //         size: font_size,
    //         color: white,
    //     });
    // }
}
