#![no_std]

pub use tato_audio as audio;
pub use tato_layout as layout;
pub use tato_math as math;
pub use tato_pad as pad;
pub use tato_rng as rng;
pub use tato_video as video;

pub mod assets;
pub use assets::*;

mod error;
pub use error::*;

pub mod graphics;

use tato_video::*;

mod tato;
pub use tato::*;

pub mod default_assets;

pub mod prelude {
    pub use crate::default_assets::*;
    pub use crate::graphics::*;
    pub use crate::*;
    pub use tato_audio::waveform::*;
    pub use tato_audio::*;
    pub use tato_layout::*;
    pub use tato_math::prelude::*;
    pub use tato_pad::*;
    pub use tato_rng::*;
    pub use tato_video::*;
}
