use crate::*;

#[derive(Debug)]
pub struct Tato {
    // Input
    pub pad: tato_pad::AnaloguePad,
    // Audio
    pub audio: tato_audio::AudioChip,
    // Video
    pub video: tato_video::VideoChip,
    pub banks: [tato_video::VideoMemory<TILE_COUNT>; BANK_COUNT],
    pub bg: BGMap<BG_LEN>,
    pub assets: Assets,
    // Internals
    pub update_time_acc: SmoothBuffer<10, f64>,
}

impl Tato {
    pub fn new(w: u16, h: u16) -> Self {
        let tato = Self {
            assets: Assets::new(),
            pad: tato_pad::AnaloguePad::default(),
            audio: tato_audio::AudioChip::default(),
            video: tato_video::VideoChip::new(w, h),
            banks: core::array::from_fn(|_| VideoMemory::new()),
            bg: BGMap::new(32, 32),
            update_time_acc: SmoothBuffer::default(),
        };

        println!("Size of Tato Engine: {:.1} Kb", size_of::<Tato>() as f32 / 1024.0);
        println!("    Tato Video: {:.1} Kb", size_of::<VideoChip>() as f32 / 1024.0);
        println!("    Tato Audio: {:.1} Kb", size_of::<AudioChip>() as f32 / 1024.0);
        println!("    Tile Banks: {:.1} Kb", size_of_val(&tato.banks) as f32 / 1024.0);
        println!("    Asset Roster: {:.1} Kb", size_of_val(&tato.assets) as f32 / 1024.0);

        tato
    }

    pub fn reset(&mut self){
        self.video.reset_all();
        self.assets.reset();
        for bank in &mut self.banks {
            bank.reset();
        }
    }

    // pub fn set_bg_size(&mut self, cols: u16, rows: u16) {
    //     self.bg.set_size(cols, rows);
    // }

    pub fn get_video_banks(&self) -> [&VideoMemory<TILE_COUNT>; BANK_COUNT] {
        core::array::from_fn(|i| &self.banks[i])
    }
}
