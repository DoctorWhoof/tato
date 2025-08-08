use tato_arena::{Arena, FillablePool, Pool};
use tato_math::{Rect, Vec2};

use crate::*;

#[derive(Debug)]
pub struct Tato {
    // pub temp: Arena<16384, u16>, // Frame-only arena allocator
    // Input
    pub pad: tato_pad::AnaloguePad,
    // Audio
    pub audio: tato_audio::AudioChip,
    // Video
    pub video: tato_video::VideoChip,
    pub banks: [tato_video::VideoMemory<TILE_COUNT>; TILE_BANK_COUNT],
    // 16Kb asset memory. Currently only stores remapped tilemaps -
    // the tiles are stored in the memory banks
    pub assets: Assets<16384>,
    // Internals
    // pub update_time_acc: SmoothBuffer<20, f64>,
    pub target_fps: u8,
    pub(crate) time: f64,
    pub(crate) delta: f32,
    pub(crate) elapsed_time: f32,
    frame_started: bool,
    frame_finished: bool,

    // #[cfg(debug_assertions)]
    debug_arena: Arena<64536, u16>, // Persistent debug arena
    // #[cfg(debug_assertions)]
    debug_strings: FillablePool<Pool<u8, u16>, u16>,
    // #[cfg(debug_assertions)]
    debug_polys: FillablePool<Pool<Vec2<i16>, u16>, u16>,
}

impl Tato {
    pub fn new(w: u16, h: u16, target_fps: u8) -> Self {
        // let temp = Arena::new();
        // #[cfg(debug_assertions)]
        let mut debug_arena = Arena::new();
        // #[cfg(debug_assertions)]
        let debug_strings = FillablePool::new(&mut debug_arena, 256).unwrap();
        // #[cfg(debug_assertions)]
        let debug_polys = FillablePool::new(&mut debug_arena, 256).unwrap();

        Self {
            // Main parts
            // temp,
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
            // #[cfg(debug_assertions)]
            debug_arena,
            // #[cfg(debug_assertions)]
            debug_strings,
            // #[cfg(debug_assertions)]
            debug_polys,
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
        // Delta timing based on 60 fps reference,
        // can still run higher or lower preserving game speed.
        let reference_frame_time = 1.0 / 60.0;
        self.delta = self.elapsed_time / reference_frame_time;

        // self.temp.clear();
        self.video.start_frame();
        self.time += elapsed as f64;
        self.elapsed_time = elapsed;
        self.frame_finished = false;
        self.frame_started = true;
        // #[cfg(debug_assertions)]
        {
            self.debug_arena.clear();
            self.debug_strings = FillablePool::new(&mut self.debug_arena, 512).unwrap();
            self.debug_polys = FillablePool::new(&mut self.debug_arena, 256).unwrap();
        }
    }

    pub fn frame_finish(&mut self) {
        if !self.frame_started {
            panic!(err!(
                "Frame must be started on each main loop iteration with 'Tato::frame_start'."
            ))
        }

        // #[cfg(debug_assertions)]
        {
            self.dash_text(&format!("Debug arena size: {} Kb", self.debug_arena.used() / 1024));
            self.dash_text(&format!("Asset arena size: {} Kb", self.assets.arena.used() / 1024));
        }

        self.frame_finished = true;
        self.frame_started = false;
    }

    pub fn dash_text(&mut self, text: &str) {
        // #[cfg(debug_assertions)]
        {
            // Set to crash if arena fails, for now. TODO: Remove unwraps, maybe return result.
            assert!(text.is_ascii());
            let handle = self.debug_arena.alloc_pool::<u8>(text.len()).unwrap();
            let pool = self.debug_arena.get_pool_mut(&handle).unwrap();
            for (i, c) in text.chars().enumerate() {
                pool[i] = c as u8;
            }
            let _ = self.debug_strings.push(&mut self.debug_arena, handle).unwrap();
        }
    }

    pub fn dash_poly(&mut self, points: &[Vec2<i16>]) {
        // #[cfg(debug_assertions)]
        {
            let handle = self.debug_arena.alloc_pool::<Vec2<i16>>(points.len()).unwrap();
            let pool = self.debug_arena.get_pool_mut(&handle).unwrap();
            pool.copy_from_slice(points);
            let _ = self.debug_polys.push(&mut self.debug_arena, handle).unwrap();
        }
    }

    pub fn dash_rect(&mut self, rect: Rect<i16>) {
        // #[cfg(debug_assertions)]
        {
            let points = [
                rect.top_left(),
                rect.top_right(),
                rect.bottom_right(),
                rect.bottom_left(),
                rect.top_left(),
            ];
            self.dash_poly(&points);
        }
    }

    #[allow(unused)]
    pub fn dash_pivot(&mut self, x: i16, y: i16, size: i16) {
        // #[cfg(debug_assertions)]
        {
            let half = size / 2;
            self.dash_poly(&[Vec2 { x: x - half, y: y - half }, Vec2 { x: x + half, y: y + half }]);
            self.dash_poly(&[Vec2 { x: x - half, y: y + half }, Vec2 { x: x + half, y: y - half }]);
        }
    }

    pub fn get_dash_text(&self) -> impl Iterator<Item = &str> + '_ {
        // #[cfg(debug_assertions)]
        {
            let debug_handles = self.debug_strings.as_slice(&self.debug_arena).unwrap_or(&[]);
            debug_handles.iter().filter_map(|handle| {
                let bytes = self.debug_arena.get_pool(handle)?;
                core::str::from_utf8(bytes).ok()
            })
        }
        // #[cfg(not(debug_assertions))]
        // {
        //     core::iter::empty()
        // }
    }

    pub fn get_dash_polys(&self) -> impl Iterator<Item = &[Vec2<i16>]> + '_ {
        // #[cfg(debug_assertions)]
        {
            let debug_handles = self.debug_polys.as_slice(&self.debug_arena).unwrap_or(&[]);
            debug_handles.iter().filter_map(|handle| self.debug_arena.get_pool(handle))
        }
        // #[cfg(not(debug_assertions))]
        // {
        //     core::iter::empty()
        // }
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
