#![no_std]
pub use tato_audio as audio;
pub use tato_layout as layout;
pub use tato_pad as pad;
pub use tato_video as video;

pub mod text;
pub mod fonts;

mod anim;
pub use anim::Anim;

pub mod prelude {
    pub use tato_audio::*;
    pub use tato_audio::waveform::*;
    pub use tato_video::*;
    pub use tato_video::prelude::*;
    pub use tato_pad::*;
    pub use crate::text::*;
}

#[derive(Debug)]
pub struct Tato {
    pub audio: tato_audio::AudioChip,
    pub video: tato_video::VideoChip,
    pub pad: tato_pad::AnaloguePad
}

impl Tato {
    pub fn new(w:u32, h:u32) -> Self {
        Self {
            audio: tato_audio::AudioChip::default(),
            video: tato_video::VideoChip::new(w, h),
            pad: tato_pad::AnaloguePad::default(),
        }
    }
}
