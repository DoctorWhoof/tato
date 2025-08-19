//! Generates the "Dashboard" UI, working in tandem with a Backend.
//! Provides a buffer of DrawOps that the Backend can render

use crate::{
    FRAME_ARENA_LEN,
    prelude::{Edge, Frame, Rect, Tato},
};
use tato_layout::Fitting;
use tato_video::{
    COLORS_PER_PALETTE, COLORS_PER_TILE, RGBA32, TILE_BANK_COUNT, TILE_COUNT, TILE_SIZE,
    VideoMemory,
};

mod ops;
pub use ops::*;

mod args;
pub use args::*;
use tato_arena::{Arena, ArenaId, Buffer, Text};

const MAX_LINES: u16 = 200;
const LINE_LEN: u16 = 80;
const OP_COUNT: u16 = 200;

// 256 tiles per bank
const MAX_TILE_PIXELS: usize =
    TILE_BANK_COUNT * TILE_SIZE as usize * TILE_SIZE as usize * TILE_COUNT as usize * 4;

/// Backend-agnostic debug UI system that generates drawing ops
#[derive(Debug)]
pub struct Dashboard {
    pub pixel_arena: Arena<MAX_TILE_PIXELS, u32>,
    pub tile_pixels: [Buffer<u8, u32>; TILE_BANK_COUNT], // one vec per bank
    pub mouse_over_text: Text,
    pub font_size: f32,
    pub console: bool,
    pub ops: Buffer<ArenaId<DrawOp>>,
    pub console_buffer: Buffer<Text, u16>,
    pub additional_text: Buffer<Text>,
    pub canvas_rect: Option<Rect<i16>>,
}

const DARKEST_GRAY: RGBA32 = RGBA32 { r: 18, g: 18, b: 18, a: 200 };
const DARK_GRAY: RGBA32 = RGBA32 { r: 32, g: 32, b: 32, a: 200 };
// const MEDIUM_GRAY: RGBA32 = RGBA32 { r: 48, g: 48, b: 48, a: 255 };

impl Dashboard {
    pub const PANEL_WIDTH: i16 = 150;
    pub const MARGIN: i16 = 10;

    pub fn new(arena: &mut Arena<FRAME_ARENA_LEN>) -> Self {
        let mut pixel_arena = Arena::<MAX_TILE_PIXELS, u32>::new(); // persistent
        let tile_pixels = core::array::from_fn(|_| {
            const CAP: u32 = TILE_COUNT as u32 * TILE_SIZE as u32 * TILE_SIZE as u32 * 4; // 4 bytes per pixel (RGBA)
            Buffer::<u8, u32>::from_fn(&mut pixel_arena, CAP, |_| 0).unwrap()
        });
        Self {
            pixel_arena,
            tile_pixels,
            mouse_over_text: Text::new(arena, LINE_LEN).unwrap(),
            font_size: 8.0,
            console: false,
            ops: Buffer::new(arena, OP_COUNT).unwrap(),
            console_buffer: Buffer::new(arena, MAX_LINES).unwrap(),
            additional_text: Buffer::new(arena, MAX_LINES).unwrap(),
            canvas_rect: None,
        }
    }

    /// Must be called at the end of each frame.
    pub fn frame_start(&mut self, arena: &mut Arena<FRAME_ARENA_LEN>) {
        self.mouse_over_text = Buffer::new(arena, LINE_LEN).unwrap();
        self.ops = Buffer::new(arena, OP_COUNT).unwrap();
        self.console_buffer = Buffer::new(arena, MAX_LINES).unwrap();
        self.additional_text = Buffer::new(arena, MAX_LINES).unwrap();
    }

    pub fn add_text(&mut self, text: &str, arena: &mut Arena<FRAME_ARENA_LEN>) {
        let text = Text::from_str(arena, text).unwrap();
        self.additional_text.push(arena, text).unwrap();
    }

    pub fn get_text_op(&mut self, text: &Text, frame: &mut Frame<i16>) -> DrawOp {
        let mut rect = Rect::default();
        let mut line_height = 0.0;
        frame.push_edge(Edge::Top, self.font_size as i16, |text_frame| {
            rect = text_frame.rect();
            line_height = self.font_size * text_frame.get_scale();
        });
        DrawOp::Text {
            text: text.clone(),
            x: rect.x,
            y: rect.y,
            size: line_height,
            color: RGBA32::WHITE,
        }
    }

    /// Generate debug UI ops
    pub fn render(&mut self, tato: &Tato, arena: &mut Arena<FRAME_ARENA_LEN>, args: DashArgs) {
        // We'll re-use as much memory as possible with a temporary arena + buffers.
        let mut temp = Arena::<16_384>::new();

        // HINT: The tricky part of dodging the borrow checker is all the closures necessary
        // to the Layout frames. Try to do as much as possible outside of the closures.

        // Add debug info
        let debug_text =
            Text::format_display(arena, "Debug mem.: {}", &[tato.debug_arena.used() / 1024], " Kb");
        self.additional_text.push(arena, debug_text.unwrap()).unwrap();

        let tiles_text = Text::format_display(
            arena,
            "Tile Debug mem.: {}",
            &[self.pixel_arena.used() / 1024],
            " Kb",
        );
        self.additional_text.push(arena, tiles_text.unwrap()).unwrap();

        let asset_text = Text::format_display(
            arena,
            "Asset mem.: {}",
            &[tato.assets.arena.used() / 1024],
            " Kb",
        );
        self.additional_text.push(arena, asset_text.unwrap()).unwrap();

        let frame_text =
            Text::format_display(arena, "Frame mem.: {}", &[arena.used() / 1024], " Kb");
        self.additional_text.push(arena, frame_text.unwrap()).unwrap();

        let fps_text = Text::format_display(arena, "fps: {:.1}", &[1.0 / tato.elapsed_time()], "");
        self.additional_text.push(arena, fps_text.unwrap()).unwrap();

        let elapsed_text =
            Text::format_display(arena, "elapsed: {:.1}", &[tato.elapsed_time() * 1000.0], "");
        self.additional_text.push(arena, elapsed_text.unwrap()).unwrap();

        let separator = Text::from_str(arena, "------------------------");
        self.additional_text.push(arena, separator.unwrap()).unwrap();

        for text in tato.iter_dash_text() {
            self.add_text(text, arena);
        }

        // Start Layout
        let screen_rect = Rect { x: 0, y: 0, w: args.screen_size.x, h: args.screen_size.y };
        let mut layout = Frame::new(screen_rect);
        layout.set_scale(args.gui_scale);
        layout.set_margin(Self::MARGIN);
        layout.set_margin(10);
        layout.set_gap(0);

        // Left panel
        {
            let mut temp_buffer = Buffer::<DrawOp>::new(&mut temp, 200).unwrap();
            layout.push_edge(Edge::Left, Self::PANEL_WIDTH, |panel| {
                panel.set_margin(5);
                // panel.set_gap(0);
                let op =
                    arena.alloc(DrawOp::Rect { rect: panel.rect(), color: DARK_GRAY }).unwrap();
                self.ops.push(arena, op).unwrap();
                let items = self.additional_text.items(arena).unwrap();
                for text in items {
                    let op = self.get_text_op(text, panel);
                    temp_buffer.push(&mut temp, op).unwrap();
                }
            });

            for op in temp_buffer.items(&temp).unwrap() {
                let handle = arena.alloc(op.clone()).unwrap();
                self.ops.push(arena, handle).unwrap()
            }
            temp.clear();
        }

        // Right panel
        {
            layout.push_edge(Edge::Right, Self::PANEL_WIDTH, |panel| {
                panel.set_margin(5);
                // panel.set_gap(0);
                panel.set_scale(args.gui_scale);
                panel.fitting = Fitting::Clamp;

                let rect_handle =
                    arena.alloc(DrawOp::Rect { rect: panel.rect(), color: DARK_GRAY });
                self.ops.push(arena, rect_handle.unwrap()).unwrap();

                // Process each video memory bank
                for bank_index in 0..TILE_BANK_COUNT {
                    // Draw each bank debug data
                    self.process_bank(bank_index, &args, tato, arena, panel);
                    // Small separator
                    panel.push_edge(Edge::Top, 5, |_separator| {});
                }
            });
        }

        // Generate ops for debug polygons
        for poly in tato.iter_dash_polys(false) {
            if poly.len() >= 2 {
                for i in 0..(poly.len() - 1) {
                    let current = poly[i];
                    let next = poly[i + 1];
                    let handle = arena
                        .alloc(DrawOp::Line {
                            x1: current.x,
                            y1: current.y,
                            x2: next.x,
                            y2: next.y,
                            color: RGBA32::WHITE,
                        })
                        .unwrap();
                    self.ops.push(arena, handle).expect("Dashboard: Can't insert GUI poly");
                }
            }
        }

        // Console
        if self.console {
            layout.push_edge(Edge::Bottom, 80, |console| {
                console.set_margin(5);
                let handle = arena
                    .alloc(DrawOp::Rect {
                        rect: console.rect(),
                        color: RGBA32 { r: 18, g: 18, b: 18, a: 230 },
                    })
                    .unwrap();
                self.ops.push(arena, handle).unwrap();
            });
        }

        // Canvas
        layout.fill(|canvas| {
            self.canvas_rect = Some(canvas.rect());
        });

        // World space polys (follow scrolling)
        if let Some(canvas_rect) = self.canvas_rect {
            for world_poly in tato.iter_dash_polys(true) {
                let scale = args.screen_size.y as f32 / args.canvas_size.y as f32;
                let scroll_x = tato.video.scroll_x as f32;
                let scroll_y = tato.video.scroll_y as f32;
                if world_poly.len() >= 2 {
                    for i in 0..(world_poly.len() - 1) {
                        let current = world_poly[i];
                        let next = world_poly[i + 1];
                        let handle = arena
                            .alloc(DrawOp::Line {
                                x1: ((current.x as f32 - scroll_x) * scale) as i16 + canvas_rect.x,
                                y1: ((current.y as f32 - scroll_y) * scale) as i16 + canvas_rect.y,
                                x2: ((next.x as f32 - scroll_x) * scale) as i16 + canvas_rect.x,
                                y2: ((next.y as f32 - scroll_y) * scale) as i16 + canvas_rect.y,
                                color: RGBA32::WHITE,
                            })
                            .unwrap();
                        self.ops.push(arena, handle).expect("Dashboard: Can't insert World poly");
                    }
                }
            }
        }

        // Generate tooltip command
        if !self.mouse_over_text.is_empty() {
            let pad = args.gui_scale as i16;
            // TODO: Need a way to calculate font size... without knowing what the font is!
            // Maybe just a multiplier, or maybe even only work with monospaced fonts?
            let width = ((self.font_size / 1.9 * self.mouse_over_text.len() as f32)
                * args.gui_scale) as i16;
            let font_size = 12.0 * args.gui_scale as f32;

            let text_x = args.mouse.x - width - 12;
            let text_y = args.mouse.y + 12;

            // Background
            let black = RGBA32 { r: 0, g: 0, b: 0, a: 255 };
            let handle = arena
                .alloc(DrawOp::Rect {
                    rect: Rect {
                        x: text_x - pad,
                        y: text_y,
                        w: width + pad + pad,
                        h: font_size as i16,
                    },
                    color: black,
                })
                .unwrap();
            self.ops.push(arena, handle).expect("Dashboard: Can't insert mouse-over rect ");

            // Text
            let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };
            let handle = arena
                .alloc(DrawOp::Text {
                    text: self.mouse_over_text.clone(),
                    x: text_x,
                    y: text_y,
                    size: font_size,
                    color: white,
                })
                .unwrap();
            self.ops.push(arena, handle).expect("Dashboard: Can't insert mouse-over text ");
        }
    }

    fn update_tile_texture(
        &mut self,
        bank_index: usize,
        bank: &VideoMemory<{ TILE_COUNT }>,
        tiles_per_row: u16,
    ) {
        // Early return for empty banks
        if bank.tile_count() == 0 {
            // self.tile_pixels[bank_index].clear();
            return;
        }

        // Calculate actual dimensions based on tile layout
        let tile_count = bank.tile_count() as u16;
        let num_rows = (tile_count + tiles_per_row - 1) / tiles_per_row; // Ceiling division

        let w = tiles_per_row as usize * TILE_SIZE as usize;
        let h = num_rows as usize * TILE_SIZE as usize;
        let expected_size = w * h * 4; // RGBA

        if expected_size != self.tile_pixels[bank_index].len() {
            // Allocate buffer with correct size
            self.tile_pixels[bank_index].resize(&mut self.pixel_arena, expected_size as u32);

            // Generate tile pixels
            let pixels = self.tile_pixels[bank_index].as_slice_mut(&mut self.pixel_arena).unwrap();

            for tile_index in 0..tile_count {
                let tile_x = tile_index % tiles_per_row;
                let tile_y = tile_index / tiles_per_row;

                for y in 0..TILE_SIZE as usize {
                    for x in 0..TILE_SIZE as usize {
                        // get color
                        let color_index =
                            bank.tiles[tile_index as usize].get_pixel(x as u8, y as u8);
                        let gray_value = color_index * 63; // Map 0-4 to 0-252
                        // get coordinates
                        let pixel_x = tile_x as usize * TILE_SIZE as usize + x;
                        let pixel_y = tile_y as usize * TILE_SIZE as usize + y;
                        let i = ((pixel_y * w as usize) + pixel_x) * 4;

                        // Seems safe for now, may need to insert a check for i < pixels.len()
                        // if I get out-of-bounds errors.
                        pixels[i] = gray_value; // R
                        pixels[i + 1] = gray_value; // G
                        pixels[i + 2] = gray_value; // B
                        pixels[i + 3] = 255; // A
                    }
                }
            }
        }
    }

    fn process_bank(
        &mut self,
        bank_index: usize,
        args: &DashArgs,
        tato: &Tato,
        arena: &mut Arena<FRAME_ARENA_LEN>,
        panel: &mut Frame<i16>,
    ) {
        let font_size = (self.font_size * args.gui_scale) as i16;
        let tiles_per_row = ((TILE_COUNT as f64).sqrt().ceil()) as u16;
        let tile_size = panel.rect().w as f32 / tiles_per_row as f32;

        let gap = args.gui_scale as i16;
        let bank = &tato.banks[bank_index];
        if bank.tile_count() == 0 && bank.color_count() == 0 && bank.sub_palette_count() == 0 {
            return;
        }

        // Bank label
        let h = font_size / args.gui_scale as i16;
        panel.push_edge(Edge::Top, h, |frame| {
            let rect = frame.rect();
            let text = Text::format_display(arena, "bank: {}", &[bank_index], "").unwrap();
            let handle = arena
                .alloc(DrawOp::Text {
                    text,
                    x: rect.x + gap,
                    y: rect.y,
                    size: font_size as f32,
                    color: RGBA32::WHITE,
                })
                .unwrap();
            self.ops.push(arena, handle).unwrap();
        });

        // Bank info
        panel.push_edge(Edge::Top, h, |frame| {
            let rect = frame.rect();
            let values =
                [bank.tile_count(), bank.color_count() as usize, bank.sub_palette_count() as usize];
            let text = Text::format_display(
                arena,
                "{} tiles, {} custom colors, {} sub-palettes",
                &values,
                "",
            )
            .unwrap();

            let handle = arena
                .alloc(DrawOp::Text {
                    text,
                    x: rect.x + gap,
                    y: rect.y,
                    size: font_size as f32 * 0.75,
                    color: RGBA32::WHITE,
                })
                .unwrap();
            self.ops.push(arena, handle).unwrap();
        });

        if bank.tile_count() == 0 {
            return;
        }

        // Color palette swatches
        panel.push_edge(Edge::Top, 8, |frame| {
            let rect = frame.rect();
            let rect_handle = arena.alloc(DrawOp::Rect { rect, color: DARKEST_GRAY }).unwrap();
            self.ops.push(arena, rect_handle).unwrap();

            let swatch_w = frame.divide_width(COLORS_PER_PALETTE as u32);
            for c in 0..COLORS_PER_PALETTE as usize {
                frame.push_edge(Edge::Left, swatch_w, |swatch| {
                    let rect = swatch.rect();
                    let color = bank.palette[c];
                    let rgba32 = RGBA32::from(color);

                    let handle = arena.alloc(DrawOp::Rect { rect, color: rgba32 }).unwrap();
                    self.ops.push(arena, handle).unwrap();

                    // Mouse hover detection
                    if rect.contains(args.mouse.x, args.mouse.y) {
                        self.mouse_over_text = Text::format_display(
                            arena,
                            "Color {} = {}, {}, {}, {}",
                            &[c as u8, color.r(), color.g(), color.b(), color.a()],
                            "",
                        )
                        .unwrap();
                    }
                });
            }
        });

        // Sub-palette swatches
        {
            let columns = 6;
            let rows = (bank.sub_palette_count() as f32 / columns as f32).ceil() as u32;
            let frame_h = (rows as i16 * 4) + 4;

            panel.push_edge(Edge::Top, frame_h, |frame| {
                frame.set_margin(1);
                frame.set_gap(1);
                let column_w = frame.divide_width(columns);
                for column in 0..columns {
                    frame.push_edge(Edge::Left, column_w, |frame_column| {
                        frame_column.set_gap(0);
                        frame_column.set_margin(1);

                        let rect = frame_column.rect();
                        let rect_handle =
                            arena.alloc(DrawOp::Rect { rect, color: DARKEST_GRAY }).unwrap();
                        self.ops.push(arena, rect_handle).unwrap();

                        let row_h = frame_column.divide_height(rows);
                        for row in 0..rows {
                            frame_column.push_edge(Edge::Top, row_h, |frame_row| {
                                // frame_row.set_gap(1);
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
                                            let swatch_rect = swatch.rect();
                                            let color_index = subp[n].0 as usize;
                                            if color_index < bank.palette.len() {
                                                let sub_rect_handle = arena
                                                    .alloc(DrawOp::Rect {
                                                        rect: swatch_rect,
                                                        color: RGBA32::from(
                                                            bank.palette[color_index],
                                                        ),
                                                    })
                                                    .unwrap();
                                                self.ops.push(arena, sub_rect_handle).unwrap();
                                            }
                                        });
                                    }

                                    // Mouse hover detection
                                    if frame_row
                                        .rect()
                                        .contains(args.mouse.x as i16, args.mouse.y as i16)
                                    {
                                        let colors = [
                                            subp_index as u8,
                                            subp[0].0,
                                            subp[1].0,
                                            subp[2].0,
                                            subp[3].0,
                                        ];
                                        self.mouse_over_text = Text::format_dbg(
                                            arena,
                                            "Sub Palette {} = [{},{},{},{}]",
                                            &colors,
                                            "",
                                        )
                                        .unwrap();
                                    }
                                }
                            });
                        }
                    });
                }
            });
        }

        // Tile visualization
        self.update_tile_texture(bank_index, bank, tiles_per_row);
        let max_row = (bank.tile_count() / tiles_per_row as usize) + 1;
        // tile_size is already in screen coordinates,
        // so I need to divide by the GUI scale.
        let tiles_height = max_row as f32 * (tile_size / args.gui_scale);

        panel.push_edge(Edge::Top, tiles_height as i16, |tiles| {
            // tiles.set_margin(0);
            // tiles.set_gap(0);
            let rect = tiles.rect();
            let rect_handle = arena.alloc(DrawOp::Rect {
                rect, //
                color: RGBA32 { r: 106, g: 96, b: 128, a: 255 },
            });
            self.ops.push(arena, rect_handle.unwrap()).unwrap();

            let texture_handle =
                arena.alloc(DrawOp::Texture { id: bank_index, rect, tint: RGBA32::WHITE }).unwrap();
            self.ops.push(arena, texture_handle).unwrap();

            // Mouse hover detection for tiles
            if rect.contains(args.mouse.x, args.mouse.y) {
                let col = ((args.mouse.x - rect.x) as f32 / tile_size) as i16;
                let row = ((args.mouse.y - rect.y) as f32 / tile_size) as i16;
                let tile_index = (row * tiles_per_row as i16) + col;
                if tile_index < bank.tile_count() as i16 {
                    self.mouse_over_text =
                        Text::format_display(arena, "Tile {}", &[tile_index], "").unwrap();
                }
            }
        });
    }
}
