use crate::*;

#[derive(Debug)]
pub struct Tato {
    pub target_fps: u8,
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
    pub update_time_acc: SmoothBuffer<20, f64>,
}

impl Tato {
    pub fn new(w: u16, h: u16, target_fps: u8) -> Self {
        Self {
            target_fps,
            assets: Assets::new(),
            pad: tato_pad::AnaloguePad::default(),
            audio: tato_audio::AudioChip::default(),
            video: tato_video::VideoChip::new(w, h),
            banks: core::array::from_fn(|_| VideoMemory::new()),
            update_time_acc: SmoothBuffer::default(),
            // arena: tato_arena::Arena::new()
        }
    }

    pub fn reset(&mut self) {
        self.video.reset_all();
        self.assets.reset();
        for bank in &mut self.banks {
            bank.reset();
        }
    }

    pub fn iter_pixels<'a>(&'a self, bg_banks: &[&'a dyn DynTilemap]) -> PixelIter<'a> {
        let video_banks: [&'a VideoMemory<256>; TILE_BANK_COUNT] =
            core::array::from_fn(|i| &self.banks[i]);
        self.video.iter_pixels(&video_banks[..], bg_banks)
    }
}
