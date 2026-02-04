use tato_audio::AudioChip;
use tato_pad::AnaloguePad;

use crate::prelude::*;

#[derive(Debug)]
pub struct Tato {
    pub paused: bool,
    pub time_scale: f32,
    // Input
    pub pad: AnaloguePad,
    // Audio
    pub audio: AudioChip,
    // Video
    pub video: VideoChip,
    // Internals
    pub target_fps: u8,
    time: u64,
    time_cache: f32, // will be pre-divided per frame to avoid divisions on every time() call
    delta: f32,
    elapsed_time: f32,
    frame_started: bool,
    frame_finished: bool,
}

impl Tato {
    pub fn new(w: u16, h: u16, target_fps: u8) -> Self {
        Self {
            paused: false,
            time_scale: 1.0,
            // assets: Assets::new(),
            pad: tato_pad::AnaloguePad::default(),
            audio: tato_audio::AudioChip::default(),
            video: tato_video::VideoChip::new(w, h),
            target_fps,
            time: 0,
            time_cache: 0.0,
            delta: 1.0,
            elapsed_time: 1.0 / target_fps as f32,
            frame_finished: true,
            frame_started: false,
        }
    }

    pub fn reset(&mut self) {
        // self.pad.clear(); // Handled by backend, otherwise it messes up "just_pressed" detection
        self.paused = false;
        self.time = 0;
        self.time_cache = 0.0;
        self.time_scale = 1.0;
        self.video.reset_all();
        // self.assets.reset();
        // for bank in &mut self.banks {
        //     bank.reset();
        // }
        self.frame_started = false;
        self.frame_finished = true;
        ();
    }

    // /// Load a Bank reference directly into a bank slot
    // pub fn load_bank(&mut self, bank_id: u8, bank: &'a Bank) {
    //     assert!((bank_id as usize) < BANK_COUNT, "Invalid bank_id");
    //     self.banks[bank_id as usize] = Some(bank);
    // }

    pub fn time(&self) -> f32 {
        self.time_cache
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
        self.video.frame_start(self.paused);

        // Quantized to a fixed interval to ensure it exactly matches
        // typical display refresh rates. Works with 60, 72, 90, 120, 180 and 240 Hz.
        // Does NOT work with PAL refresh rates. Sorry, not sorry.
        const ELAPSED_QUANT_SIZE: f32 = 1.0 / 1440.0; // 3X 120Hz, 6X 60Hz
        // Will not attempt to delta-correct if frame rate is ridiculously low, since
        // that could cause weird physics (like teleporting through objects)
        const ELAPSED_MAX: f32 = 1.0 / 15.0;
        let elapsed = tato_math::quantize(elapsed, ELAPSED_QUANT_SIZE) //
            .clamp(ELAPSED_QUANT_SIZE, ELAPSED_MAX);

        if self.paused {
            self.delta = 0.0;
        } else {
            let reference_frame_time = 1.0 / 60.0;
            self.elapsed_time = elapsed * self.time_scale;
            self.delta = self.elapsed_time / reference_frame_time;
            let elapsed_microseconds = (elapsed * self.time_scale * 1_000_000.0) as u64;
            self.time += elapsed_microseconds;
            self.time_cache = self.time as f32 / 1_000_000.0;
        }

        self.frame_finished = false;
        self.frame_started = true;
        // self.pad.copy_current_to_previous_state();
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

    // pub fn iter_pixels<T>(&'a self, tilemaps: &[&'a T]) -> PixelIter<'a>
    // where
    //     &'a T: Into<TilemapRef<'a>>,
    // {
    //     let default_bank = self.banks[0].expect("Tato banks mmust have at least one valid bank at index 0");
    //     let video_banks: [&'a Bank; BANK_COUNT] =
    //         core::array::from_fn(|i| match self.banks[i] {
    //             Some(bank) => bank,
    //             None => default_bank
    //         });
    //     self.video.iter_pixels(&video_banks[..], tilemaps)
    // }

    pub fn iter_pixels<'a, T>(&'a self, banks: &'a [Bank], tilemaps: &'a [&'a T]) -> PixelIter<'a>
    where
        &'a T: Into<TilemapRef<'a>>,
    {
        self.video.iter_pixels(banks, tilemaps)
    }

}
