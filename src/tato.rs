use core::fmt::Debug;

use tato_arena::{Arena, Buffer, Slice, Text};
use tato_math::{Rect, Vec2};

use crate::*;

const DEBUG_STR_COUNT: u16 = 100;
const DEBUG_STR_LINE_LEN: u16 = 80;
const DEBUG_POLY_COUNT: u16 = 100;

#[derive(Debug)]
pub struct Tato {
    // Input
    pub pad: tato_pad::AnaloguePad,

    // Audio
    pub audio: tato_audio::AudioChip,

    // Video
    pub video: tato_video::VideoChip,
    pub banks: [tato_video::VideoMemory<TILE_COUNT>; TILE_BANK_COUNT],
    // 16Kb asset memory. Currently only stores remapped tilemaps -
    // the tiles are stored directly in the memory banks
    pub assets: Assets<16384>,

    // Internals
    pub target_fps: u8,
    pub(crate) time: f64,
    pub(crate) delta: f32,
    pub(crate) elapsed_time: f32,
    frame_started: bool,
    frame_finished: bool,

    pub debug_arena: Arena<FRAME_ARENA_LEN>, // Debug arena, cleared on every frame start
    debug_strings: Buffer<Text>,
    debug_polys: Buffer<Slice<Vec2<i16>>>,
    debug_polys_world: Buffer<Slice<Vec2<i16>>>,
}

impl Tato {
    pub fn new(w: u16, h: u16, target_fps: u8) -> Self {
        let mut debug_arena = Arena::new();
        let debug_polys = Buffer::new(&mut debug_arena, DEBUG_POLY_COUNT).unwrap();
        let debug_polys_world = Buffer::new(&mut debug_arena, DEBUG_POLY_COUNT).unwrap();
        let debug_strings =
            Buffer::text_multi_buffer(&mut debug_arena, DEBUG_STR_COUNT, DEBUG_STR_LINE_LEN, true)
                .unwrap();

        Self {
            assets: Assets::new(),
            pad: tato_pad::AnaloguePad::default(),
            audio: tato_audio::AudioChip::default(),
            video: tato_video::VideoChip::new(w, h),
            banks: core::array::from_fn(|_| VideoMemory::new()),
            target_fps,
            time: 0.0,
            delta: 1.0,
            elapsed_time: 1.0 / target_fps as f32,
            frame_finished: true,
            frame_started: false,
            debug_arena,
            debug_strings,
            debug_polys,
            debug_polys_world,
        }
    }

    pub fn reset(&mut self) {
        self.time = 0.0;
        self.video.reset_all();
        self.assets.reset();
        for bank in &mut self.banks {
            bank.reset();
        }
        self.frame_started = false;
        self.frame_finished = true;
        ();
    }

    pub fn time(&self) -> f64 {
        self.time
    }

    pub fn elapsed_time(&self) -> f32 {
        self.elapsed_time as f32
    }

    /// Delta time multiplier where 1.0 means "running at target frame rate".
    /// It assumes a fixed reference where speed 1.0 = 1 pixel per frame at 60 fps.
    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn frame_start(&mut self, elapsed: f32) {
        if !self.frame_finished {
            panic!(err!(
                "Frame must be finished on each main loop iteration with 'Tato::frame_finish'."
            ))
        }
        // Delta timing based on 60 fps reference (delta = 1.0)
        // can still run higher or lower while preserving game speed.
        let reference_frame_time = 1.0 / 60.0;
        self.delta = self.elapsed_time / reference_frame_time;

        self.video.start_frame();
        self.time += elapsed as f64;
        self.elapsed_time = elapsed;
        self.frame_finished = false;
        self.frame_started = true;

        self.debug_arena.clear();
        // Re-inits buffers. (if I just clear, all arena handles are invalidated!)
        self.debug_polys = Buffer::new(&mut self.debug_arena, DEBUG_POLY_COUNT).unwrap();
        self.debug_polys_world = Buffer::new(&mut self.debug_arena, DEBUG_POLY_COUNT).unwrap();
        self.debug_strings = Buffer::text_multi_buffer(
            &mut self.debug_arena,
            DEBUG_STR_COUNT,
            DEBUG_STR_LINE_LEN,
            true,
        )
        .unwrap();
    }

    pub fn frame_finish(&mut self) {
        if !self.frame_started {
            panic!(err!(
                "Frame must be started on each main loop iteration with 'Tato::frame_start'."
            ))
        }

        self.frame_finished = true;
        self.frame_started = false;
    }

    // --------------------- Dashboard ---------------------

    /// Sends text messages to the dashboard buffer. Unfortunately, formatting
    /// those messages requires std library. For basic, no_std formatting check
    /// "dash_dbg" below.
    pub fn dash_text(&mut self, text: &str) {
        // Set to crash if arena fails, for now. TODO: Remove unwraps, maybe return result.
        assert!(text.is_ascii());
        let text = Text::from_str(&mut self.debug_arena, text).unwrap();
        self.debug_strings.push(&mut self.debug_arena, text).unwrap();
    }

    /// Allows basic text formatting when sending text to the dashboard
    pub fn dash_dbg<T>(&mut self, message: &str, values: &[T])
    where
        T: Debug,
    {
        // Set to crash if arena fails, for now. TODO: Remove unwraps, maybe return result.
        let handle = Text::format_dbg(&mut self.debug_arena, message, values).unwrap();
        self.debug_strings.push(&mut self.debug_arena, handle).unwrap();
    }

    /// Sends an open polygon to the dashboard (to close, simply ensure the last
    /// point matches the first). If "world_space" is true, poly will be resized
    /// and translated to match canvas size and scroll values. If not, it will
    /// be drawn like a gui.
    pub fn dash_poly(&mut self, points: &[Vec2<i16>], world_space: bool) {
        let handle = self.debug_arena.alloc_slice::<Vec2<i16>>(points.len() as u16).unwrap();
        let slice = self.debug_arena.get_slice_mut(&handle).unwrap();
        slice.copy_from_slice(points);
        if world_space {
            let _ = self.debug_polys_world.push(&mut self.debug_arena, handle).unwrap();
        } else {
            let _ = self.debug_polys.push(&mut self.debug_arena, handle).unwrap();
        }
    }

    /// Convenient way to send a rect as a poly to the dashboard.
    pub fn dash_rect(&mut self, rect: Rect<i16>, world_space: bool) {
        let points = [
            rect.top_left(),
            rect.top_right(),
            rect.bottom_right(),
            rect.bottom_left(),
            rect.top_left(),
        ];
        self.dash_poly(&points, world_space);
    }

    /// Convenient way to send a point as an "x" to the dashboard.
    pub fn dash_pivot(&mut self, x: i16, y: i16, size: i16, world_space: bool) {
        let half = size / 2;
        self.dash_poly(
            &[Vec2 { x: x - half, y: y - half }, Vec2 { x: x + half, y: y + half }],
            world_space,
        );
        self.dash_poly(
            &[Vec2 { x: x - half, y: y + half }, Vec2 { x: x + half, y: y - half }],
            world_space,
        );
    }

    // --------------------- Iterators ---------------------

    pub fn iter_dash_text(&self) -> impl Iterator<Item = &str> {
        let debug_handles = self.debug_strings.as_slice(&self.debug_arena).unwrap_or(&[]);
        debug_handles.iter().filter_map(|handle| {
            let bytes = self.debug_arena.get_slice(&handle.slice)?;
            core::str::from_utf8(bytes).ok()
        })
    }

    pub fn iter_dash_polys(&self, world_space: bool) -> impl Iterator<Item = &[Vec2<i16>]> {
        let debug_handles = if world_space {
            self.debug_polys_world.as_slice(&self.debug_arena).unwrap_or(&[])
        } else {
            self.debug_polys.as_slice(&self.debug_arena).unwrap_or(&[])
        };
        debug_handles.iter().filter_map(|handle| self.debug_arena.get_slice(handle))
    }

    pub fn iter_pixels<'a, T>(&'a self, bg_banks: &[&'a T]) -> PixelIter<'a>
    where
        &'a T: Into<TilemapRef<'a>>,
    {
        let video_banks: [&'a VideoMemory<256>; TILE_BANK_COUNT] =
            core::array::from_fn(|i| &self.banks[i]);
        self.video.iter_pixels(&video_banks[..], bg_banks)
    }
}
