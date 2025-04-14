#![no_std]

pub use tato_pad as pad;
pub use tato_video as video;
pub use tato_layout as layout;

pub mod prelude {
    pub use crate::backend::*;
    pub use tato_pad::*;
    pub use tato_video::*;
}

pub mod backend {
    use tato_pad::AnaloguePad;
    use tato_video::VideoChip;

    pub trait BackendVideo {
        fn new_window(vid: &VideoChip) -> Self;
        fn frame_start(&mut self, vid: &VideoChip);
        fn frame_update(&mut self, vid: &VideoChip);
        fn frame_finish(&mut self, vid: &VideoChip);
        fn gamepad(&self) -> AnaloguePad;
        fn quit_requested(&self) -> bool;
        fn elapsed(&self) -> f64;
        fn time(&self) -> f64;
    }
}
