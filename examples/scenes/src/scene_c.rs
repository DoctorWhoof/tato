use crate::*;
use tato::video::prelude::*;

#[derive(Debug)]
pub struct SceneC {
    smiley: TileID,
    counter: u64,
}

impl SceneC {
    pub fn new(vid: &mut VideoChip) -> Self {
        let _bg = vid.new_tile(&TILE_CROSSHAIRS);
        let tile = vid.new_tile(&TILE_SOLID);
        let smiley = vid.new_sprite(2, &data::LARGE_SPRITE);

        for row in 0..vid.bg.columns {
            for col in 0..vid.bg.rows {
                vid.bg.set_flags(col, row, TileFlags::default().with_fg());
            }
        }

        for id in 0..16 as u8 {
            vid.bg.set_tile(BgBundle {
                col: id,
                row: 0,
                tile_id: tile,
                flags: TileFlags::from(PaletteID(id % 16)).with_fg(),
            });
            vid.set_palette(PaletteID(id), [BG_COLOR, ColorID(id), BG_COLOR, BG_COLOR]);
        }

        SceneC { smiley, counter: 0 }
    }

    pub fn update(&mut self, vid: &mut VideoChip, app: BackendState) -> Option<SceneChange> {
        vid.start_frame();

        // Draw the sprite directly, no Entity
        let mut offset = 0.0;
        for x in 0..16 {
            let time = (self.counter as f32 / 60.0) + offset;
            let wave = (time.sin() + 1.0) / 2.0;
            let y = (wave * 180.0) as i16;
            offset += 0.1;
            vid.draw_sprite(DrawBundle {
                x: x * 16,
                y,
                id: self.smiley,
                flags: PaletteID(x as u8).into(),
            });
        }

        self.counter += 1;

        // Input
        if app.pad.is_just_pressed(Button::Menu) {
            Some(SceneChange::A)
        } else {
            None
        }
    }
}
