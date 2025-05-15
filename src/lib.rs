use tato_audio::AudioChip;
// #![no_std]
pub use tato_audio as audio;
pub use tato_layout as layout;
pub use tato_pad as pad;
pub use tato_video as video;

use tato_video::*;

pub mod tilesets;
pub mod text;

mod anim;
pub use anim::Anim;

mod tileset;
pub use tileset::*;

mod tile_bank;
pub use tile_bank::*;

mod rect;
pub use rect::*;

pub mod prelude {
    pub use crate::text::*;
    pub use tato_audio::waveform::*;
    pub use tato_audio::*;
    pub use tato_pad::*;
    pub use tato_video::*;
    // pub use tato_video::*;
}

const TILE_BANK_COUNT:usize = 2;
const TILE_MAP_COUNT:usize = 2;

#[derive(Debug)]
pub struct Tato {
    pub audio: tato_audio::AudioChip,
    pub video: tato_video::VideoChip,
    pub pad: tato_pad::AnaloguePad,
    pub tiles: TileBank<TILE_BANK_COUNT>,
    pub maps: [Tilemap<BG_LEN>; TILE_MAP_COUNT],
}

impl Tato {
    pub fn new(w: u16, h: u16) -> Self {
        let tato = Self {
            audio: tato_audio::AudioChip::default(),
            video: tato_video::VideoChip::new(w, h),
            pad: tato_pad::AnaloguePad::default(),
            tiles: TileBank::default(),
            maps: core::array::from_fn(|_| Tilemap::new(64, 48)),
        };

        println!(
            "Size of Tato Engine: {:.1} Kb",
            size_of::<Tato>() as f32 / 1024.0
        );
        println!(
            "    Tato Video: {:.1} Kb",
            size_of::<VideoChip>() as f32 / 1024.0
        );
        println!(
            "    Tato Audio: {:.1} Kb",
            size_of::<AudioChip>() as f32 / 1024.0
        );
        println!(
            "    Tile Banks: {:.1} Kb",
            size_of_val(&tato.tiles) as f32 / 1024.0
        );
        println!(
            "    Tile Maps: {:.1} Kb",
            size_of_val(&tato.maps) as f32 / 1024.0
        );

        tato
    }

    pub fn map_refs(&self) -> [&Tilemap<BG_LEN>; TILE_MAP_COUNT] {
        core::array::from_fn(|i|{
            &self.maps[i]
        })
    }
}
