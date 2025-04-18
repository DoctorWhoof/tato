#![no_std]

pub use tato_audio as audio;
pub use tato_layout as layout;
pub use tato_pad as pad;
pub use tato_video as video;

pub mod prelude {
    pub use tato_audio::*;
    pub use tato_audio::waveform::*;
    pub use tato_video::*;
    pub use tato_pad::*;
}

#[derive(Debug)]
pub struct Tato {
    pub audio: tato_audio::AudioChip,
    pub video: tato_video::VideoChip,
    pub pad: tato_pad::AnaloguePad
}
