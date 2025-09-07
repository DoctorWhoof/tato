use crate::{prelude::CharacterSet, *};

#[derive(Debug)]
pub struct Tato {
    pub paused: bool,
    // Input
    pub pad: tato_pad::AnaloguePad,
    // Audio
    pub audio: tato_audio::AudioChip,
    // Video
    pub video: tato_video::VideoChip,
    pub banks: [tato_video::VideoMemory<TILE_COUNT>; TILE_BANK_COUNT],
    pub character_set: CharacterSet,
    // 16Kb asset memory. Currently only stores remapped tilemaps -
    // the tiles are stored directly in the memory banks
    pub assets: Assets<16384>,
    // Internals
    pub target_fps: u8,
    time: f64,
    delta: f32,
    elapsed_time: f32,
    frame_started: bool,
    frame_finished: bool,
}

impl Tato {
    pub fn new(w: u16, h: u16, target_fps: u8) -> Self {
        Self {
            paused: false,
            assets: Assets::new(),
            pad: tato_pad::AnaloguePad::default(),
            audio: tato_audio::AudioChip::default(),
            video: tato_video::VideoChip::new(w, h),
            banks: core::array::from_fn(|_| VideoMemory::new()),
            character_set: CharacterSet::Long,
            target_fps,
            time: 0.0,
            delta: 1.0,
            elapsed_time: 1.0 / target_fps as f32,
            frame_finished: true,
            frame_started: false,
        }
    }

    pub fn reset(&mut self) {
        self.paused = false;
        self.pad.clear();
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
        self.video.frame_start();

        if self.paused {
            self.delta = 0.0;
        } else {
            let reference_frame_time = 1.0 / 60.0;
            self.delta = self.elapsed_time / reference_frame_time;
            self.elapsed_time = elapsed;
            self.time += elapsed as f64;
        }

        self.frame_finished = false;
        self.frame_started = true;
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

    // --------------------- Iterators ---------------------

    pub fn iter_pixels<'a, T>(&'a self, bg_banks: &[&'a T]) -> PixelIter<'a>
    where
        &'a T: Into<TilemapRef<'a>>,
    {
        let video_banks: [&'a VideoMemory<256>; TILE_BANK_COUNT] =
            core::array::from_fn(|i| &self.banks[i]);
        self.video.iter_pixels(&video_banks[..], bg_banks)
    }
}
