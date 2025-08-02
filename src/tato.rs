use crate::*;

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
    // the tiles are stored in the memory banks
    pub assets: Assets<16384>,
    // Internals
    // pub update_time_acc: SmoothBuffer<20, f64>,
    pub target_fps: u8,
    pub(crate) time: f64,
    pub(crate) delta: f32,
    pub(crate) elapsed_time: f32,
}

impl Tato {
    pub fn new(w: u16, h: u16, target_fps: u8) -> Self {
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
        }
    }

    pub fn reset(&mut self) {
        self.time = 0.0;
        self.video.reset_all();
        self.assets.reset();
        for bank in &mut self.banks {
            bank.reset();
        }
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

    pub fn start_frame(&mut self, elapsed: f32) {
        self.video.start_frame();
        self.time += elapsed as f64;
        self.elapsed_time = elapsed;
        // Delta timing based on 60 fps reference,
        // can still run higher or lower preserving game speed.
        let reference_frame_time = 1.0 / 60.0;
        self.delta = self.elapsed_time / reference_frame_time;
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
