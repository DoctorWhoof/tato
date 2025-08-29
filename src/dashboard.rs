//! Generates the "Dashboard" UI, working in tandem with a Backend.
//! Provides a buffer of DrawOps that the Backend can render, as well as a buffer of Console commands.

use crate::arena::{Arena, ArenaId, ArenaResult, Buffer, Text};
use crate::layout::Fitting;
use crate::prelude::*;
use crate::video::{
    COLORS_PER_PALETTE, COLORS_PER_TILE, RGBA32, TILE_BANK_COUNT, TILE_COUNT, TILE_SIZE,
    VideoMemory,
};

mod command;
pub use command::*;

mod key;
pub use key::*;

mod ops;
pub use ops::*;

mod gui_console;
mod gui_draw_polys;
mod gui_draw_tooltip;
mod gui_input;
mod gui_text;
mod gui_video;

const MAX_LINES: u32 = 200;
const OP_COUNT: u32 = 200;

// 256 tiles per bank
const MAX_TILE_PIXELS: usize =
    TILE_BANK_COUNT * TILE_SIZE as usize * TILE_SIZE as usize * TILE_COUNT as usize * 4;
const FIXED_ARENA_LEN: usize = MAX_TILE_PIXELS + (32 * 1024);

/// Backend-agnostic debug UI system that generates drawing ops
#[derive(Debug)]
pub struct Dashboard {
    pub font_size: f32,
    pub gui_scale: f32,
    fixed_arena: Arena<FIXED_ARENA_LEN, u32>,
    ops: Buffer<ArenaId<DrawOp, u32>, u32>,
    mouse_over_text: Text<u32>,
    additional_text: Buffer<Text<u32>, u32>,
    tile_pixels: [Buffer<u8, u32>; TILE_BANK_COUNT], // one vec per bank
    last_frame_arena_use: usize,
    console_buffer: Buffer<Text>,
    console_line_buffer: Buffer<u8>,
    canvas_rect: Option<Rect<i16>>,
    // input related
    display_debug_info: bool,
    display_console: bool,
}

pub const PANEL_WIDTH: i16 = 150;
pub const MARGIN: i16 = 10;
const DARKEST_GRAY: RGBA32 = RGBA32 { r: 18, g: 18, b: 18, a: 200 };
const DARK_GRAY: RGBA32 = RGBA32 { r: 32, g: 32, b: 32, a: 200 };

impl Dashboard {
    /// Creates a new Dashboard where LEN is the memory available to its
    /// temporary memory buffer, in bytes.
    pub fn new<const LEN: usize>(frame_arena: &mut Arena<LEN, u32>) -> TatoResult<Self> {
        let mut fixed_arena = Arena::<FIXED_ARENA_LEN, u32>::new(); // persistent
        let tile_pixels = {
            // 4 bytes per pixel (RGBA)
            const CAP: u32 = TILE_COUNT as u32 * TILE_SIZE as u32 * TILE_SIZE as u32 * 4;
            // Messy, but allows using '?' per bank
            let mut result: [core::mem::MaybeUninit<Buffer<u8, u32>>; TILE_BANK_COUNT] =
                unsafe { core::mem::MaybeUninit::uninit().assume_init() };

            for i in 0..TILE_BANK_COUNT {
                result[i] = core::mem::MaybeUninit::new(Buffer::<u8, u32>::from_fn(
                    &mut fixed_arena,
                    CAP,
                    |_| 0,
                )?);
            }
            unsafe { core::mem::transmute(result) }
        };
        let console_buffer = Buffer::new(&mut fixed_arena, MAX_LINES)?;
        let console_line_buffer = Buffer::new(&mut fixed_arena, COMMAND_MAX_LEN as u32).unwrap();

        let ops = Buffer::new(frame_arena, OP_COUNT)?;
        let additional_text = Buffer::new(frame_arena, MAX_LINES)?;

        Ok(Self {
            font_size: 8.0,
            gui_scale: 2.0,
            // frame_arena,
            fixed_arena,
            tile_pixels,
            mouse_over_text: Text::default(),
            ops,
            additional_text,
            last_frame_arena_use: 0,
            // console_display: false,
            console_line_buffer,
            console_buffer,
            canvas_rect: None,
            // last_key_received: Key::None,
            display_debug_info: true,
            display_console: false,
        })
    }

    /// A reference to the pixel buffer used to debug tile pixels, if
    /// the desired bank contains one
    pub fn tile_pixels(&self, bank_index: usize) -> Option<&[u8]> {
        let pixel_buffer = self.tile_pixels.get(bank_index)?;
        pixel_buffer.as_slice(&self.fixed_arena).ok()
    }

    // /// The space allocated to draw the canvas
    // pub fn canvas_rect(&self) -> Option<Rect<i16>> {
    //     self.canvas_rect
    // }

    /// An iterator with every DrawOp processed so far
    pub fn draw_ops<'a, const LEN: usize>(
        &self,
        frame_arena: &'a Arena<LEN, u32>,
    ) -> ArenaResult<impl Iterator<Item = &'a DrawOp>> {
        self.ops.items(frame_arena).map(|iter| iter.filter_map(|id| frame_arena.get(id).ok()))
    }

    /// Must be called at the beginning of each frame, clears buffers.
    pub fn frame_start<const LEN: usize>(&mut self, frame_arena: &mut Arena<LEN, u32>) {
        self.ops = Buffer::new(frame_arena, OP_COUNT).unwrap();
        self.mouse_over_text = Text::default(); // Text unallocated, essentially same as "None"
        self.additional_text = Buffer::new(frame_arena, MAX_LINES).unwrap();
    }

    /// Creates an internal temp_arena-allocated Text object, stores its ID
    /// in a list so it can be drawn when "render" is called.
    pub fn push_text<const LEN: usize>(&mut self, text: &str, frame_arena: &mut Arena<LEN, u32>) {
        let text = Text::from_str(frame_arena, text).unwrap();
        self.additional_text.push(frame_arena, text).unwrap();
    }

    /// Generates a Text DrawOp with coordinates relative to a layout Frame
    /// (will push a new edge from the Top in the frame to reserve room for the text)
    pub fn get_text_op(&self, text: Text<u32>, frame: &mut Frame<i16>) -> DrawOp {
        let mut rect = Rect::default();
        let mut line_height = 0.0;
        frame.push_edge(Edge::Top, self.font_size as i16, |text_frame| {
            rect = text_frame.rect();
            line_height = self.font_size * text_frame.get_scale();
        });
        DrawOp::Text {
            text,
            x: rect.x,
            y: rect.y,
            size: line_height,
            color: RGBA32::WHITE,
        }
    }

    /// Generate debug UI ops
    // HINT: The tricky part of dodging the borrow checker is all the closures necessary
    // to the Layout frames. Try to do as much as possible outside of the closures.
    pub fn render<const LEN: usize>(
        &mut self,
        frame_arena: &mut Arena<LEN, u32>,
        backend: &mut impl Backend,
        tato: &Tato,
    ) {
        // Start Layout
        let screen_size = backend.get_screen_size();
        let screen_rect = Rect { x: 0, y: 0, w: screen_size.x, h: screen_size.y };
        let mut layout = Frame::new(screen_rect);
        layout.set_scale(self.gui_scale);
        layout.set_margin(MARGIN);
        layout.set_margin(10);
        layout.set_gap(3);

        // We need to process
        self.process_input(backend);
        if !self.display_debug_info {
            return;
        }

        // Panels have their own modules, for organization
        self.process_text_panel(&mut layout, frame_arena, backend, tato);
        self.process_video_panel(&mut layout, frame_arena, backend, tato);
        self.process_console(&mut layout, frame_arena);

        // Canvas
        layout.fill(|canvas| {
            // Calculate canvas placement within this frame, taking aspect ratio into account.
            // The canvas texture can then be drawn by the backend using this rectangle.
            let (rect, _scale) = canvas_rect_and_scale(canvas.rect(), tato.video.size(), false);
            self.canvas_rect = Some(rect);
            backend.set_canvas_rect(Some(rect));
        });

        // Copy tile pixels from dashboard to GPU textures
        for bank_index in 0..TILE_BANK_COUNT {
            // texture ID = bank_index
            if let Some(pixels) = self.tile_pixels(bank_index) {
                if !pixels.is_empty() {
                    backend.update_texture(bank_index, pixels);
                }
            }
        }

        // Draw additional items over everything
        self.draw_polys(frame_arena, &tato, backend);
        self.draw_tooltip(frame_arena, backend);

        backend.set_additional_draw_ops(self.ops.clone());
        // if let Ok(ops) = self.ops.as_slice(frame_arena) {
        //     backend.push_ops(frame_arena, ops);
        // }

        // Acquire at the very end so it's accurate (although with a 1 frame delay)
        self.last_frame_arena_use = frame_arena.used();
    }
}
