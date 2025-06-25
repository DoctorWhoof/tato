use crate::*;
use tato::prelude::*;

#[derive(Debug)]
pub struct SceneB {
    player: Entity,
    smileys: [Entity; 64],
}

impl SceneB {
    pub fn new(t: &mut Tato) -> Self {
        // Center view
        let x = t.video.max_x() / 2;
        let y = t.video.max_y() / 2;
        // Shrinks the viewport by 8 pixels on each edge
        t.video.set_viewport(8, 8, 224, 164);

        // Colors
        t.video.bg_color = RGBA12::DARK_GREEN;
        let palette_bg = t
            .banks[0]
            .push_subpalette([DARK_GREEN, GREEN, DARK_GREEN, DARK_GREEN]);
        let palette_smiley = t.banks[0].push_subpalette([BG_COLOR, YELLOW, BLACK, BLACK]);
        let palette_cycler = t.banks[0].push_subpalette([BG_COLOR, WHITE, BLACK, BLACK]);
        let _tileset = t.new_tileset(0, DEFAULT_TILESET);
        let tile = TILE_SMILEY;

        t.bg.set_size(32, 24);

        for cell in &mut t.bg.cells {
            cell.id = tile;
            cell.flags = palette_bg.into();
        }

        Self {
            player: Entity {
                x: x as f32,
                y: y as f32,
                tile,
                flags: palette_cycler.into(),
            },
            smileys: core::array::from_fn(|_| Entity {
                // Will test wrapping of large f32 value into i16
                // using "wrap_width" and "wrap_height"
                x: rand::random_range(0.0..255.0), // - 32_000.0,
                y: rand::random_range(0.0..255.0), // + 32_000.0,
                tile,
                flags: palette_smiley.into(),
            }),
        }
    }

    pub fn update(&mut self, t: &mut Tato, app: BackendState) -> Option<SceneChange> {
        t.video.start_frame();
        let speed = 1.0;

        // Input
        if app.pad.is_down(Button::Left) {
            self.player.x -= speed;
        } else if app.pad.is_down(Button::Right) {
            self.player.x += speed;
        }
        if app.pad.is_down(Button::Up) {
            self.player.y -= speed;
        } else if app.pad.is_down(Button::Down) {
            self.player.y += speed;
        }

        // Draw!
        t.banks[0].color_cycle(self.player.flags.palette(), 1, 1, 15);

        // TODO: center_on(sprite) function
        for (_i, entity) in self.smileys.iter_mut().enumerate() {
            entity.x -= speed;
            entity.y += speed;
            t.video.draw_sprite(DrawBundle {
                x: entity.x as i16,
                y: entity.y as i16,
                id: entity.tile,
                flags: entity.flags,
            });
        }

        t.video.draw_sprite(DrawBundle {
            x: self.player.x as i16,
            y: self.player.y as i16,
            id: self.player.tile,
            flags: self.player.flags,
        });

        if app.pad.is_just_pressed(Button::Start) {
            t.video.wrap_sprites = !t.video.wrap_sprites;
        }

        if app.pad.is_just_pressed(Button::Menu) {
            Some(SceneChange::C)
        } else {
            None
        }
    }
}
