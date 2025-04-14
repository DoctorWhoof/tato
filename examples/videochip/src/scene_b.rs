use crate::*;
use tato::video::prelude::*;

#[derive(Debug)]
pub struct SceneB {
    player: Entity,
    smileys: [Entity; 64],
}

impl SceneB {
    pub fn new(vid: &mut VideoChip) -> Self {
        // Center view
        let x = vid.max_x() / 2;
        let y = vid.max_y() / 2;
        // Shrinks the viewport by 8 pixels on each edge, creating the
        // illusion that sprites go outside the border - in reality they are
        // culled as soon as they hit the left or top border
        vid.set_viewport(8, 8, 224, 164);

        // Colors
        vid.bg_color = DARK_GREEN;
        let _palette_bg = vid.push_palette([BG_COLOR, GREEN, BLACK, BLACK]);
        let palette_smiley = vid.push_palette([BG_COLOR, YELLOW, BLACK, BLACK]);
        let palette_cycler = vid.push_palette([BG_COLOR, WHITE, BLACK, BLACK]);

        // Since we're only defining one tile and it is tile 0, it will automatically
        // be used in the BG, since by default the BG tiles are all set to zero.
        let tile = vid.new_tile(8, 8, &data::SMILEY);

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
                x: rand::random_range(0.0 .. 255.0), // - 32_000.0,
                y: rand::random_range(0.0 .. 255.0), // + 32_000.0,
                tile,
                flags: palette_smiley.into(),
            }),
        }
    }

    pub fn update(&mut self, vid: &mut VideoChip, app: BackendState) -> Option<SceneChange> {
        vid.start_frame();
        let speed = 1.0;
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
        vid.color_cycle(self.player.flags.palette(), 1, 1, 15);

        // TODO: center_on(sprite) function
        for (_i, entity) in self.smileys.iter_mut().enumerate() {
            entity.x -= speed;
            entity.y += speed;
            vid.draw_sprite(DrawBundle {
                x: entity.x as i16,
                y: entity.y as i16,
                id: entity.tile,
                flags: entity.flags,
            });
        }

        vid.draw_sprite(DrawBundle {
            x: self.player.x as i16,
            y: self.player.y as i16,
            id: self.player.tile,
            flags: self.player.flags,
        });

        if app.pad.is_just_pressed(Button::Start) {
            vid.wrap_sprites = !vid.wrap_sprites;
            // println!("Wrap: {}", vid.wrap_sprites);
        }

        if app.pad.is_just_pressed(Button::Menu) {
            Some(SceneChange::C)
        } else {
            None
        }
    }
}
