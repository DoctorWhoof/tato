use crate::*;
use tato::default_assets::*;
use tato::prelude::*;

#[derive(Debug)]
pub struct SceneB {
    player: Entity,
    smileys: [Entity; 64],
}

impl SceneB {
    pub fn new(t: &mut Tato, banks: &mut [Bank], state: &mut State) -> TatoResult<Self> {
        t.video.reset_all();
        t.video.fg_tile_bank = 0;
        t.video.bg_tile_bank = 1;

        // Center view
        let x = t.video.max_x() / 2;
        let y = t.video.max_y() / 2;
        // Shrinks the viewport by 8 pixels on each edge
        t.video.set_viewport(8, 8, 224, 164);
        t.video.wrap_sprites = true;

        // Colors
        banks[0].reset();
        banks[0].colors.load_default();

        banks[1].reset();
        banks[1].colors.load_default();

        t.video.bg_color = RGBA12::BLACK;
        t.video.crop_color = RGBA12::DARK_GREEN;

        // let _tileset = t.push_tileset(0, DEFAULT_TILESET)?;
        let tile = TILE_SMILEY.id;

        // Set BG cells to use mappings
        for (i, cell) in state.bg.cells.iter_mut().enumerate() {
            let color = (i % 3) as u8 + 9;
            cell.id = tile;
            cell.flags = TileFlags::default();
            cell.colors = [0, 1, color, 0].into()
        }

        Ok(Self {
            player: Entity {
                x: x as f32,
                y: y as f32,
                tile,
                flags: TileFlags::default(),
                colors: [0, 1, 8, 3].into(),
            },
            smileys: core::array::from_fn(|_| Entity {
                x: rand::random_range(0.0..255.0),
                y: rand::random_range(0.0..255.0),
                tile,
                flags: TileFlags::default(),
                colors: [0, 9, 10, 11].into(),
            }),
        })
    }

    pub fn update(&mut self, t: &mut Tato) -> Option<SceneChange> {
        let speed = 1.0;

        // Input
        if t.pad.is_down(Button::Left) {
            self.player.x -= speed;
        } else if t.pad.is_down(Button::Right) {
            self.player.x += speed;
        }
        if t.pad.is_down(Button::Up) {
            self.player.y -= speed;
        } else if t.pad.is_down(Button::Down) {
            self.player.y += speed;
        }

        // TODO: center_on(sprite) function
        for (_i, entity) in self.smileys.iter_mut().enumerate() {
            entity.x -= speed;
            entity.y += speed;
            t.video.draw_fg_tile(DrawBundle {
                x: entity.x as i16,
                y: entity.y as i16,
                id: entity.tile,
                flags: entity.flags,
                colors: entity.colors,
            });
        }

        t.video.draw_fg_tile(DrawBundle {
            x: self.player.x as i16,
            y: self.player.y as i16,
            id: self.player.tile,
            flags: self.player.flags,
            colors: self.player.colors,
        });

        if t.pad.is_just_pressed(Button::Start) {
            t.video.wrap_sprites = !t.video.wrap_sprites;
        }

        if t.pad.is_just_pressed(Button::Menu) { Some(SceneChange::C) } else { None }
    }
}
