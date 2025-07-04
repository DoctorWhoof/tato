use crate::*;

#[derive(Debug)]
pub struct Tato<'a> {
    // Input
    pub pad: tato_pad::AnaloguePad,
    // Audio
    pub audio: tato_audio::AudioChip,
    // Video
    pub video: tato_video::VideoChip,
    pub banks: [tato_video::VideoMemory<TILE_COUNT>; BANK_COUNT],
    pub bg: Option<&'a mut dyn DynamicBGMap>,
    pub assets: Assets,
    // Internals
    pub update_time_acc: SmoothBuffer<10, f64>,
}

impl<'a> Tato<'a> {
    pub fn new(w: u16, h: u16) -> Self {
        Self {
            bg: None,
            assets: Assets::new(),
            pad: tato_pad::AnaloguePad::default(),
            audio: tato_audio::AudioChip::default(),
            video: tato_video::VideoChip::new(w, h),
            banks: core::array::from_fn(|_| VideoMemory::new()),
            update_time_acc: SmoothBuffer::default(),
        }
    }

    pub fn reset(&mut self) {
        self.video.reset_all();
        self.assets.reset();
        for bank in &mut self.banks {
            bank.reset();
        }
    }

    pub fn get_video_banks(&self) -> [&VideoMemory<TILE_COUNT>; BANK_COUNT] {
        core::array::from_fn(|i| &self.banks[i])
    }
}
