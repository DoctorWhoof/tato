use crate::*;

#[derive(Debug)]
pub struct Tato {
    pub assets: AssetRoster,
    pub pad: tato_pad::AnaloguePad,
    pub audio: tato_audio::AudioChip,
    pub video: tato_video::VideoChip,
    pub banks: [tato_video::VideoMemory<TILE_COUNT, BG_LEN>; BANK_COUNT],
    pub update_time_acc: SmoothBuffer<10, f64>,
}

impl Tato {
    pub fn new(w: u16, h: u16) -> Self {
        let tato = Self {
            assets: AssetRoster::new(),
            pad: tato_pad::AnaloguePad::default(),
            audio: tato_audio::AudioChip::default(),
            video: tato_video::VideoChip::new(w, h),
            banks: core::array::from_fn(|_| VideoMemory::new()),
            update_time_acc: SmoothBuffer::default(),
        };

        println!("Size of Tato Engine: {:.1} Kb", size_of::<Tato>() as f32 / 1024.0);
        println!("    Tato Video: {:.1} Kb", size_of::<VideoChip>() as f32 / 1024.0);
        println!("    Tato Audio: {:.1} Kb", size_of::<AudioChip>() as f32 / 1024.0);
        println!("    Tile Banks: {:.1} Kb", size_of_val(&tato.banks) as f32 / 1024.0);

        tato
    }

    pub fn reset(&mut self){
        self.video.reset_all();
        self.assets.reset();
        for bank in &mut self.banks {
            bank.reset();
        }
    }

    pub fn set_bg_size(&mut self, bank_id: u8, cols: u16, rows: u16) {
        self.banks[bank_id as usize].bg.set_size(cols, rows);
    }

    pub fn get_video_banks(&self) -> [&VideoMemory<TILE_COUNT, CELLS_PER_BANK>; BANK_COUNT] {
        core::array::from_fn(|i| &self.banks[i])
    }
}
