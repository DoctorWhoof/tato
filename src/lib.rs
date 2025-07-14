#![no_std]

use smooth_buffer::SmoothBuffer;
pub use tato_audio as audio;
pub use tato_layout as layout;
pub use tato_pad as pad;
pub use tato_video as video;

pub mod assets;
pub use assets::*;

use tato_video::*;

pub mod graphics;

mod tato;
pub use tato::*;

pub mod default_assets;

pub mod prelude {
    pub use crate::*;
    pub use crate::graphics::*;
    pub use crate::default_assets::*;
    pub use tato_audio::waveform::*;
    pub use tato_audio::*;
    pub use tato_layout::*;
    pub use tato_pad::*;
    pub use tato_video::*;
    pub use tato_math::prelude::*;
}

pub const CELLS_PER_BANK:usize = 1024; // Maximum BG Map size per bank
