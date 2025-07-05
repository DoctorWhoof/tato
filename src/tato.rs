use crate::*;

#[derive(Debug)]
pub struct Tato<'a> {
    // Input
    pub pad: tato_pad::AnaloguePad,
    // Audio
    pub audio: tato_audio::AudioChip,
    // Video
    pub video: tato_video::VideoChip,
    pub banks: [tato_video::VideoMemory<TILE_COUNT>; TILE_BANK_COUNT],
    pub bg: [Option<&'a mut dyn DynamicBGMap>; 8],
    pub assets: Assets,
    // Internals
    pub update_time_acc: SmoothBuffer<10, f64>,
}

impl<'a> Tato<'a> {
    pub fn new(w: u16, h: u16) -> Self {
        Self {
            bg: core::array::from_fn(|_| None),
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

    pub fn iter_pixels(&self) -> PixelIter {
        let bg_banks = core::array::from_fn(|i| {
            if let Some(bg) = &self.bg[i] { Some(*bg as &dyn DynamicBGMap) } else { None }
        });
        let video_banks = core::array::from_fn(|i| &self.banks[i]);
        self.video.iter_pixels(video_banks, bg_banks)
    }
}
