// #![no_std]

use smooth_buffer::SmoothBuffer;
pub use tato_audio as audio;
pub use tato_layout as layout;
pub use tato_pad as pad;
pub use tato_video as video;

pub mod assets;
pub use assets::*;

use tato_audio::*;
use tato_video::*;

pub mod graphics;

mod tato;
pub use tato::*;

mod tile_set;
use tile_set::*;

pub mod tilesets;

pub mod prelude {
    pub use crate::*;
    pub use crate::graphics::*;
    pub use crate::tile_set::*;
    pub use crate::tilesets::*;
    pub use tato_audio::waveform::*;
    pub use tato_audio::*;
    pub use tato_layout::*;
    pub use tato_pad::*;
    pub use tato_video::*;
}

pub const BANK_COUNT: usize = 2;
pub const CELLS_PER_BANK:usize = 1024; // Maximum BG Map size per bank
