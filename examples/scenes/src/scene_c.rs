use crate::*;
use tato::video::prelude::*;

#[derive(Debug)]
pub struct SceneC {
    smiley: TileID,
    counter: u64,
}

impl SceneC {
    pub fn new(t: &mut Tato) -> Self {
        let _bg = t.tiles.new_tile(&TILE_CROSSHAIRS);
        let tile = t.tiles.new_tile(&TILE_SOLID);
        let smiley = t.tiles.new_tile(&data::SMILEY);

        for row in 0..t.video.bg.columns {
            for col in 0..t.video.bg.rows {
                t.video
                    .bg
                    .set_flags(col, row, TileFlags::default().with_fg());
            }
        }

        for id in 0..16 as u8 {
            t.video.bg.set_tile(BgBundle {
                col: id,
                row: 0,
                tile_id: tile,
                flags: TileFlags::from(PaletteID(id % 16)).with_fg(),
            });
            t.video
                .set_palette(PaletteID(id), [BG_COLOR, ColorID(id), BG_COLOR, BG_COLOR]);
        }

        SceneC { smiley, counter: 0 }
    }

    pub fn update(&mut self, t: &mut Tato, app: BackendState) -> Option<SceneChange> {
        t.video.start_frame();

        // Draw the sprite directly, no Entity
        let mut offset = 0.0;
        for x in 0..16 {
            let time = (self.counter as f32 / 60.0) + offset;
            let wave = ((time * 4.0).sin() + 1.0) / 2.0;
            let y = (wave * 8.0) as i16 + 60;
            offset += 0.1;
            t.video.draw_sprite(
                DrawBundle {
                    x: x * 8,
                    y,
                    id: self.smiley,
                    flags: PaletteID(x as u8).into(),
                },
                &t.tiles.tiles,
            );
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
