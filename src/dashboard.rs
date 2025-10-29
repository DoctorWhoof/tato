//! Generates the "Dashboard" UI, working in tandem with a Backend.
//! Provides a buffer of DrawOps that the Backend can render, as well as a buffer of Console commands.

use crate::arena::{Arena, ArenaId, ArenaRes, Buffer, Text};
use crate::layout::Fitting;
use crate::prelude::*;
use crate::video::{
    COLORS_PER_PALETTE, COLORS_PER_TILE, RGBA32, TILE_BANK_COUNT, TILE_COUNT, TILE_SIZE, VideoBank,
};

mod command;
pub use command::*;

mod key;
pub use key::*;

mod ops;
pub use ops::*;
use tato_arena::{RingBuffer, Slice};

mod gui_banks;
mod gui_console;
mod gui_draw_polys;
mod gui_draw_tooltip;
mod gui_input;
mod gui_text;

// The Fixed arena is never cleared - this may need to changed when
// I dynamically update the tiles! (i.e. pop() and load())
const FIXED_ARENA_LEN: usize = MAX_TILE_PIXELS + (32 * 1024);
// 256 tiles per bank
const MAX_TILE_PIXELS: usize =
    TILE_BANK_COUNT * TILE_SIZE as usize * TILE_SIZE as usize * TILE_COUNT as usize * 4;
const COMMAND_MAX_LEN: u32 = 100;
const COMMAND_MAX_ARGS: usize = 8;

// Temp Debug Arena
// This is necessary since DrawOps need to be processed, and can't be read
// (Text, Vec2, etc) and written (DrawOp) to the same arena at the same time.
const DEBUG_LEN: usize = 8 * 1024;
const CONSOLE_HISTORY: u32 = 10;
const OP_COUNT: u32 = 500;
const DEBUG_STR_COUNT: u32 = 100;
const DEBUG_POLY_COUNT: u32 = 100;

/// Backend-agnostic debug UI system that generates drawing ops
#[derive(Debug)]
pub struct Dashboard {
    pub font_size: f32,
    pub gui_scale: f32,
    // Storage
    fixed_arena: Arena<FIXED_ARENA_LEN, u32>, //  Never cleared
    debug_arena: Arena<DEBUG_LEN>,            // Cleared on every frame
    // State
    display_debug_info: bool,
    display_console: bool,
    // Console
    console_buffer: RingBuffer<[u8; COMMAND_MAX_LEN as usize]>,
    console_line_buffer: Buffer<u8>,
    console_latest_command: Option<Command>,
    canvas_rect: Option<Rect<i16>>,
    // Debug data
    last_frame_arena_use: usize,
    last_frame_draw_op_count: usize,
    mouse_over_text: Text,
    ops: Buffer<ArenaId<DrawOp, u32>, u32>,
    debug_text: Buffer<Text, u32>,
    debug_polys_world: Buffer<Slice<Vec2<i16>>>,
    debug_polys_gui: Buffer<Slice<Vec2<i16>>>,
    tile_pixels: [Buffer<u8, u32>; TILE_BANK_COUNT], // one vec per bank
}

pub const PANEL_WIDTH: i16 = 150;
pub const MARGIN: i16 = 10;
const DARKEST_GRAY: RGBA32 = RGBA32 { r: 18, g: 18, b: 18, a: 200 };
const DARK_GRAY: RGBA32 = RGBA32 { r: 32, g: 32, b: 32, a: 200 };

impl Dashboard {
    /// Creates a new Dashboard where LEN is the memory available to its
    /// temporary memory buffer, in bytes.
    pub fn new() -> TatoResult<Self> {
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
        let console_buffer = RingBuffer::new(&mut fixed_arena, CONSOLE_HISTORY)?;
        let console_line_buffer = Buffer::new(&mut fixed_arena, COMMAND_MAX_LEN).unwrap();

        Ok(Self {
            font_size: 8.0,
            gui_scale: 2.0,
            // frame_arena,
            fixed_arena,
            tile_pixels,
            last_frame_arena_use: 0,
            last_frame_draw_op_count: 0,
            // console_display: false,
            console_latest_command: None,
            canvas_rect: None,
            // last_key_received: Key::None,
            display_debug_info: true,
            display_console: false,
            // Fixed arena data (not cleared on every frame)
            // Shared frame arena data
            console_line_buffer,
            console_buffer,
            mouse_over_text: Text::default(),
            // Debug data. Will be arena allocated per frame, but not use the frame arena
            // to avoid reading and writing to the same arena when saving DrawOps
            ops: Buffer::default(),
            debug_arena: Arena::new(),
            debug_text: Buffer::default(),
            debug_polys_world: Buffer::default(),
            debug_polys_gui: Buffer::default(),
        })
    }

    /// A reference to the pixel buffer used to debug tile pixels, if
    /// the desired bank contains one
    pub fn tile_pixels(&self, bank_index: usize) -> Option<&[u8]> {
        let pixel_buffer = self.tile_pixels.get(bank_index)?;
        pixel_buffer.as_slice(&self.fixed_arena).ok()
    }

    /// An iterator with every DrawOp processed so far. DrawOps must be stored
    /// in a external frame arena, since they are shared with the Backend.
    pub fn draw_ops<'a, const LEN: usize>(
        &self,
        frame_arena: &'a Arena<LEN>,
    ) -> ArenaRes<impl Iterator<Item = &'a DrawOp>> {
        self.ops
            .items(frame_arena) //
            .map(|iter| iter.filter_map(|id| frame_arena.get(id).ok()))
    }

    /// If a console command has been processed this frame it is returned here.
    pub fn process_console_line<'a, F, const LEN: usize>(
        &'a mut self,
        frame_arena: &mut Arena<LEN>,
        func: F,
    ) where
        F: FnOnce(Command) -> Option<&'a [u8]>,
    {
        if let Some(command) = &self.console_latest_command {
            let temp = ['?' as u8];
            let reply = func(command.clone()).unwrap_or_else(|| &temp);
            // Get only the valid portion of command.data (up to first null byte)
            let command_len =
                command.data.iter().position(|&b| b == 0).unwrap_or(command.data.len());
            let command_slice = &command.data[..command_len];
            // Use frame_arena for temporary text creation, then copy to fixed array
            let joined_text =
                Text::join_bytes(frame_arena, &[command_slice, b" -> ", reply]).unwrap();
            let bytes = joined_text.as_slice(frame_arena).unwrap();
            let line_with_reply: [u8; COMMAND_MAX_LEN as usize] =
                core::array::from_fn(|i| if i >= bytes.len() { 0 } else { bytes[i] });

            self.console_buffer.push(&mut self.fixed_arena, line_with_reply).unwrap();
        }
    }

    /// Creates an internal temp_arena-allocated Text object, stores its ID
    /// in a list so it can be drawn when "render" is called.
    pub fn str(&mut self, text: &str) {
        let text = Text::from_str(&mut self.debug_arena, text).unwrap();
        self.debug_text.push(&mut self.debug_arena, text).unwrap();
    }

    /// Allows basic text formatting when sending text to the dashboard
    pub fn debug<T>(&mut self, message: &str, values: &[T], tail: &str)
    where
        T: core::fmt::Debug,
    {
        // Set to crash if arena fails, for now. TODO: Remove unwraps, maybe return result.
        let handle = Text::format_dbg(&mut self.debug_arena, message, values, tail).unwrap();
        self.debug_text.push(&mut self.debug_arena, handle).unwrap();
    }

    /// Allows basic text formatting when sending text to the dashboard
    pub fn display<T>(&mut self, message: &str, values: &[T], tail: &str)
    where
        T: core::fmt::Display,
    {
        // Set to crash if arena fails, for now. TODO: Remove unwraps, maybe return result.
        let handle = Text::format_display(&mut self.debug_arena, message, values, tail).unwrap();
        self.debug_text.push(&mut self.debug_arena, handle).unwrap();
    }

    /// Sends an open polygon to the dashboard (to close, simply ensure the last
    /// point matches the first). If "world_space" is true, poly will be resized
    /// and translated to match canvas size and scroll values. If not, it will
    /// be drawn like a gui.
    pub fn poly(&mut self, points: &[Vec2<i16>], world_space: bool) {
        let handle = self.debug_arena.alloc_slice::<Vec2<i16>>(points.len()).unwrap();
        let slice = self.debug_arena.get_slice_mut(&handle).unwrap();
        slice.copy_from_slice(points);
        if world_space {
            self.debug_polys_world.push(&mut self.debug_arena, handle).unwrap();
        } else {
            self.debug_polys_gui.push(&mut self.debug_arena, handle).unwrap();
        }
    }

    /// Convenient way to send a rect as a poly to the dashboard.
    pub fn rect(&mut self, rect: Rect<i16>, world_space: bool) {
        let points = [
            rect.top_left(),
            rect.top_right(),
            rect.bottom_right(),
            rect.bottom_left(),
            rect.top_left(),
        ];
        self.poly(&points, world_space);
    }

    /// Convenient way to send a point as an "x" to the dashboard.
    pub fn pivot(&mut self, x: i16, y: i16, size: i16, world_space: bool) {
        let half = size / 2;
        self.poly(
            &[Vec2 { x: x - half, y: y - half }, Vec2 { x: x + half, y: y + half }],
            world_space,
        );
        self.poly(
            &[Vec2 { x: x - half, y: y + half }, Vec2 { x: x + half, y: y - half }],
            world_space,
        );
    }

    /// Must be called at the beginning of each frame, after the Backend has
    /// started its own frame
    pub fn frame_start<const LEN: usize>(
        &mut self,
        frame_arena: &mut Arena<LEN>,
        backend: &mut impl Backend,
    ) {
        self.console_latest_command = None;
        self.last_frame_draw_op_count = self.ops.len();

        // Shared frame arena data
        self.ops = Buffer::new(frame_arena, OP_COUNT).unwrap();
        self.mouse_over_text = Text::default(); // Text unallocated, essentially same as "None"

        // Internal debug arena data
        self.debug_arena.clear();
        self.debug_text = Buffer::new(&mut self.debug_arena, DEBUG_STR_COUNT).unwrap();
        self.debug_polys_world = Buffer::new(&mut self.debug_arena, DEBUG_POLY_COUNT).unwrap();
        self.debug_polys_gui = Buffer::new(&mut self.debug_arena, DEBUG_POLY_COUNT).unwrap();

        // Input
        let text_input = self.display_console && self.display_debug_info;
        backend.set_game_input(!text_input); // if console is active, no gameplay input allowed
        self.process_input(backend);
    }

    /// Generate debug UI Draw Ops before presenting them via the Backend.
    pub fn frame_present<const LEN: usize>(
        &mut self,
        frame_arena: &mut Arena<LEN>,
        backend: &mut impl Backend,
        tato: &Tato,
    ) {
        if !self.display_debug_info {
            return;
        }

        // Start Layout
        let screen_size = backend.get_screen_size();
        let screen_rect = Rect { x: 0, y: 0, w: screen_size.x, h: screen_size.y };
        let mut layout = Frame::new(screen_rect);
        layout.set_scale(self.gui_scale);
        layout.set_margin(MARGIN);
        // layout.set_margin(10);
        layout.set_gap(3);

        // Panels have their own modules, for organization
        self.process_text_panel(&mut layout, frame_arena, backend, tato);
        self.process_video_banks_panel(&mut layout, frame_arena, backend, tato);
        self.process_console(tato, &mut layout, frame_arena);

        // Canvas
        layout.fill(|canvas| {
            // Calculate canvas placement within this frame, taking aspect ratio into account.
            // The canvas texture can then be drawn by the backend using this rectangle.
            let (rect, _scale) = canvas_rect_and_scale(canvas.rect(), tato.video.size(), false);
            self.canvas_rect = Some(rect);
            backend.set_canvas_rect(Some(rect));
            {
                let mid_x = screen_size.x / 2;
                let mid_y = screen_size.y / 2;
                self.poly(&[Vec2::new(rect.left(), mid_y), Vec2::new(rect.right(), mid_y)], false);
                self.poly(&[Vec2::new(mid_x, rect.top()), Vec2::new(mid_x, rect.bottom())], false);
            }
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

        self.draw_polys(frame_arena, &tato);
        self.draw_tooltip(frame_arena, backend);
        backend.set_additional_draw_ops(self.ops.clone());

        // Acquire arena usage info at the very end
        // so it's accurate (although with a 1 frame delay)
        self.last_frame_arena_use = frame_arena.used();
    }
}
