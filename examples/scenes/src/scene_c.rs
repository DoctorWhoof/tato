use crate::*;
use tato::prelude::*;

#[derive(Debug)]
pub struct SceneC {
    smiley: TileID,
    counter: u64,
}

impl SceneC {
    pub fn new(t: &mut Tato, state: &mut State) -> TatoResult<Self> {
        let _tileset = t.push_tileset(0, DEFAULT_TILESET)?;
        let solid = TILE_SOLID;
        let cross = TILE_CROSSHAIRS;
        let smiley = TILE_SMILEY;

        t.video.bg_color = RGBA12::GRAY;

        for col in 0..state.bg.columns() {
            for row in 0..state.bg.rows() {
                state.bg.set_cell(BgOp {
                    col,
                    row,
                    tile_id: cross,
                    flags: TileFlags::from(PaletteID(1)).with_fg(),
                });
            }
        }

        for id in 0..16 as u8 {
            state.bg.set_cell(BgOp {
                col: id as u16,
                row: 0,
                tile_id: solid,
                flags: TileFlags::from(PaletteID(id % 16)).with_fg(),
            });
            t.banks[0].set_subpalette(PaletteID(id), [BG_COLOR, ColorID(id), BG_COLOR, BG_COLOR]);
        }

        Ok(SceneC { smiley, counter: 0 })
    }

    pub fn update(&mut self, t: &mut Tato, state: &mut State) -> Option<SceneChange> {
        // Draw the sprite directly, no Entity
        let mut offset = 0.0;
        for x in 0..16 {
            let time = (self.counter as f32 / 60.0) + offset;
            let wave = ((time * 4.0).sin() + 1.0) / 2.0;
            let y = (wave * 8.0) as i16 + 60;
            offset += 0.1;
            t.video.draw_fg_tile(DrawBundle {
                x: x * 8,
                y,
                id: self.smiley,
                flags: PaletteID(x as u8).into(),
            });
        }

        self.counter += 1;

        // Input
        if state.pad.is_just_pressed(Button::Menu) { Some(SceneChange::A) } else { None }
    }
}
