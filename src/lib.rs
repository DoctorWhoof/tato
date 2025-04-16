#![no_std]

pub use tato_audio as audio;
pub use tato_layout as layout;
pub use tato_pad as pad;
pub use tato_video as video;

pub mod prelude {
    // pub use crate::backend::*;
    pub use tato_audio::*;
    pub use tato_video::*;
    pub use tato_pad::*;
}

use tato_audio::*;
use tato_video::*;
use tato_pad::*;

#[derive(Debug)]
pub struct Tato {
    pub audio: AudioChip,
    pub video: VideoChip,
    pub pad: AnaloguePad
}


// pub mod backend {
//     use tato_audio::AudioChip;
//     use tato_pad::AnaloguePad;
//     use tato_video::VideoChip;

//     pub trait TatoBackend {
//         fn new_window(video_chip: Option<&VideoChip>, audio_chip: Option<&AudioChip>) -> Self;
//         fn frame_start(&mut self, vid: &VideoChip);
//         fn frame_update(&mut self, vid: &VideoChip);
//         fn frame_finish(&mut self, vid: &VideoChip);
//         fn gamepad(&self) -> AnaloguePad;
//         fn quit_requested(&self) -> bool;
//         fn elapsed(&self) -> f64;
//         fn time(&self) -> f64;
//         fn audio_update_buffer(&mut self, audio: &AudioChip);
//     }
// }
