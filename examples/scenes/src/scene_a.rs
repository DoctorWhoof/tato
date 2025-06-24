use crate::*;
use core::array::from_fn;
use tato::prelude::*;

const SMILEY_COUNT: usize = 32;

#[derive(Debug)]
pub struct SceneA {
    pub player: Entity,
    smileys: [Entity; SMILEY_COUNT],
    movement_start: f32, // will be used to store the time when the player starts moving
}

impl SceneA {
    // Initialize and retuns a new scene
    // pub fn new(video: &mut VideoChip, tiles: &mut TileBank) -> Self {
    pub fn new(t: &mut Tato) -> Self {
        t.video.bg_color = RGBA12::new(3, 1, 1, 7);
        t.video.wrap_bg = false;
        t.video.wrap_sprites = false;
        t.video.bg_tile_bank = 1;

        // Palette test - defines BG palette with a golden tint!
        t.banks[1].palette = [
            RGBA12::new(0, 0, 0, 0),
            RGBA12::new(2, 1, 1, 7),
            RGBA12::new(3, 1, 1, 7),
            RGBA12::new(4, 2, 1, 7),
            RGBA12::new(4, 2, 1, 7),
            RGBA12::new(5, 2, 2, 7),
            RGBA12::new(5, 3, 2, 7),
            RGBA12::new(5, 4, 2, 7),
            RGBA12::new(5, 4, 3, 7),
            RGBA12::new(6, 4, 3, 7),
            RGBA12::new(6, 4, 4, 7),
            RGBA12::new(6, 5, 4, 7),
            RGBA12::new(6, 6, 4, 7),
            RGBA12::new(6, 6, 4, 7),
            RGBA12::new(6, 6, 5, 7),
            RGBA12::new(6, 6, 5, 7),
        ];

        // Procedural BG Palettes. Each PaletteID matches a ColorID
        for i in 0..t.banks[1].palette.len() {
            let _ = t
                .banks[0]
                .push_subpalette([BG_COLOR, ColorID(i as u8), BLACK, BLACK]);
            let _ = t
                .banks[1]
                .push_subpalette([BG_COLOR, ColorID(i as u8), BLACK, BLACK]);
        }

        // Define new tiles
        let _tileset_fg = t.new_tileset(0, DEFAULT_TILESET).unwrap();
        let _tileset_bg = t.new_tileset(1, DEFAULT_TILESET).unwrap();
        // let _tileset = t.add_tileset(0, &TILESET_DEFAULT).unwrap();
        let smiley = TILE_SMILEY;
        let arrow = TILE_ARROW;

        // Set BG tiles
        t.bg.set_size(28, 28);
        // let t.bg = &mut t.banks[0].bg;
        for col in 0..t.bg.columns {
            for row in 0..t.bg.rows {
                // Calculate palette ID based on coordinates, limits to 14 indices
                let index = (col + row) % 14;
                // Adds 2 to avoid colors 0 and 1 in the BG
                let adjusted_palette = PaletteID(2 + index as u8);

                t.bg.set_cell(BgOp {
                    col,
                    row,
                    tile_id: arrow,
                    flags: TileFlags::from(adjusted_palette).with_rotation(),
                });
            }
        }

        // Pre-generate smileys array
        let mut smileys: [Entity; SMILEY_COUNT] = from_fn(|i| {
            Entity {
                x: rand::random_range(0..t.bg.width() as i16 - TILE_SIZE as i16) as f32,
                y: rand::random_range(0..t.bg.height() as i16 - TILE_SIZE as i16) as f32,
                tile: smiley,
                flags: PaletteID(4 + (i % 12) as u8).into(), // Avoids grayscale in default palette
            }
        });

        // Sort smileys by y position only
        smileys.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        // Store initial state and return
        Self {
            player: Entity {
                x: (t.video.width() / 2) as f32,
                y: (t.video.height() / 2) as f32,
                tile: arrow,
                flags: PaletteID(0).into(),
            },
            smileys,
            movement_start: 0.0,
        }
    }

    // Process the scene on each frame
    pub fn update(&mut self, t: &mut Tato, app: BackendState) -> Option<SceneChange> {
        t.video.start_frame();

        // ------------------------------ Input ------------------------------

        if app.pad.is_just_pressed(Button::Start) {
            t.video.wrap_sprites = !t.video.wrap_sprites;
            t.video.wrap_bg = !t.video.wrap_bg;
        }

        // Increase speed if any "face" button (A,B,X,Y) is down
        // let speed = if app.pad.is_any_down(AnyButton::Face) {
        let speed = if app.pad.is_down(Button::A) {
            30.0 * app.elapsed as f32
        } else {
            120.0 * app.elapsed as f32
        };

        // Ensures animation always starts from phase = 0.0;
        if app.pad.is_any_just_pressed(AnyButton::Direction) {
            self.movement_start = app.time as f32;
        }

        // Player Movement
        let is_walking = {
            let (mut vel_x, mut vel_y) = (0.0, 0.0);
            if app.pad.is_down(Button::Left) {
                vel_x = -speed;
                self.player.flags.rotate_left();
            } else if app.pad.is_down(Button::Right) {
                vel_x = speed;
                self.player.flags.rotate_right();
            }
            if app.pad.is_down(Button::Up) {
                vel_y = -speed;
                self.player.flags.rotate_up();
            } else if app.pad.is_down(Button::Down) {
                vel_y = speed;
                self.player.flags.rotate_down();
            }

            if vel_x != 0.0 || vel_y != 0.0 {
                self.player.x += vel_x;
                self.player.y += vel_y;
                true
            } else {
                false
            }
        };

        // ------------------------------ Draw ------------------------------

        // Adjust scroll and palette before drawing characters! (immediate mode)
        let target_x = (self.player.x - (t.video.width() as f32 / 2.0)).floor() as i16;
        let target_y = (self.player.y - (t.video.height() as f32 / 2.0)).floor() as i16;
        t.video.scroll_x = target_x;
        t.video.scroll_y = target_y;

        t.banks[0].color_cycle(self.player.flags.palette(), 1, 1, 15);

        // let t.bg = &mut t.banks[0].bg;
        for col in 0..t.bg.columns {
            for row in 0..t.bg.rows {
                let Some(mut flags) = t.bg.get_flags(col, row) else {
                    continue;
                };
                flags.set_rotation(self.player.flags.is_rotated());
                flags.set_flip_x(self.player.flags.is_flipped_x());
                flags.set_flip_y(self.player.flags.is_flipped_y());
                t.bg.set_flags(col, row, flags);
            }
        }

        // Draw shadows first (lowest priority).
        let mut sprite_shadow = |entity: &Entity| {
            t.video.draw_sprite(DrawBundle {
                x: entity.x as i16,
                y: entity.y as i16,
                id: entity.tile,
                // Remember, we generated palettes that match the color indices
                flags: entity.flags.with_palette(PaletteID(BLACK.0)),
            });
        };

        for entity in &self.smileys {
            sprite_shadow(entity);
        }
        sprite_shadow(&self.player);

        // Draw Sprites with hover animation
        let mut sprite_hover = |entity: &Entity, phase: f32, speed: f32, height: f32| {
            let hover = ((phase * speed).sin() + 1.0) * height;
            t.video.draw_sprite(DrawBundle {
                x: (entity.x - 1.0) as i16,
                y: (entity.y - 1.0 - hover) as i16,
                id: entity.tile,
                flags: entity.flags,
            });
        };

        for entity in &self.smileys {
            // passing x as phase gives us out-of-sync motion
            let hover_phase = entity.x + app.time as f32;
            sprite_hover(entity, hover_phase, 6.0, 1.5);
        }

        // Player goes in front. Drawing a sprite last means it has highest priority!
        let hover_phase = app.time as f32 - self.movement_start;
        let hover_speed = if is_walking { 24.0 } else { 4.0 };
        let hover_height = if is_walking { 2.0 } else { 1.5 };
        sprite_hover(&self.player, hover_phase, hover_speed, hover_height);

        // Flashing Smiley at the origin
        t.video.draw_sprite(DrawBundle {
            x: 0,
            y: 0,
            id: TILE_SMILEY,
            flags: TileFlags::default(), // Player palette is zero
        });

        // ------------------- Return mode switch request -------------------

        if app.pad.is_just_pressed(Button::Menu) {
            Some(SceneChange::B)
        } else {
            None
        }
    }
}
