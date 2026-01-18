use crate::*;
use tato::default_assets::*;
use tato::prelude::*;

#[derive(Debug)]
pub struct SceneC {
    smiley: TileID,
    counter: u64,
}

static mut LINE: u16 = 0;

impl SceneC {
    pub fn new(t: &mut Tato, banks: &mut [Bank], state: &mut State) -> TatoResult<Self> {
        t.video.reset_all();
        // let _tileset = t.push_tileset(0, DEFAULT_TILESET)?;
        let _solid = TILE_SOLID;
        let cross = TILE_CROSSHAIRS;
        let smiley = TILE_SMILEY;

        banks[0].reset();
        banks[1].reset();
        banks[0].colors.load_default();
        t.video.bg_color = RGBA12::GRAY;

        for col in 0..state.bg.columns() as i16 {
            for row in 0..state.bg.rows() as i16 {
                state.bg.set_op(BgOp {
                    col,
                    row,
                    cell: Cell {
                        id: cross.id,
                        flags: TileFlags::default().with_fg(),
                        color_mapping: 0,
                        group: 0,
                    },
                });
            }
        }
        // Color mappings
        for n in 0..COLOR_MAPPING_COUNT as usize {
            banks[0].colors.mapping[n][2] = n as u8;
        }

        // BG color raster effects
        t.video.irq_line = Some(|iter, _chip, _tilemap| {
            let y = iter.y();
            let line = unsafe { y.wrapping_add(LINE) };
            let color = &mut iter.bg_color;

            let scaled_line = line / 4;

            color.set_r((scaled_line % 4) as u8 + 3);
            color.set_g(((scaled_line.wrapping_add(1)) % 3) as u8 + 2);
            color.set_b(((scaled_line.wrapping_add(2)) % 3) as u8 + 4);
        });

        Ok(SceneC { smiley: smiley.id, counter: 0 })
    }

    pub fn update(&mut self, t: &mut Tato, _banks: &mut [Bank]) -> Option<SceneChange> {
        if t.video.frame_number() % 4 == 0 {
            unsafe {
                LINE = LINE.wrapping_sub(1);
            }
        }

        // Draw the sprite directly, no Entity
        let mut offset = 0.0;
        for x in 0..16 {
            let time = (self.counter as f32 / 60.0) + offset;
            let wave = ((time * 4.0).sin() + 1.0) / 2.0;
            let y = (wave * 16.0) as i16 + 80;
            offset += 0.1;
            t.video.draw_fg_tile(DrawBundle {
                x: x * 15,
                y,
                id: self.smiley,
                flags: TileFlags::default(),
                color_mapping: x as u8,
            });
        }

        self.counter += 1;

        // Input
        if t.pad.is_just_pressed(Button::Menu) { Some(SceneChange::A) } else { None }
    }
}
