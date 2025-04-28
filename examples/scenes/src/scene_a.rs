use crate::*;
use core::array::from_fn;
use tato::pad::*;
use tato::video::prelude::*;

const SMILEY_COUNT: usize = 1000;

#[derive(Debug)]
pub struct SceneA {
    pub player: Entity,
    smileys: [Entity; SMILEY_COUNT],
    movement_start: f32, // will be used to store the time when the player starts moving
}

impl SceneA {
    // Initialize and retuns a new scene
    pub fn new(vid: &mut VideoChip) -> Self {
        vid.bg_color = BLACK;
        vid.wrap_bg = false;
        vid.wrap_sprites = false;
        // vid.set_viewport(8, 8, 240, 176);
        vid.set_crop_x(16);
        vid.set_crop_y(16);

        // Palette test - defines BG palette with a golden tint!
        vid.bg_palette = [
            Color9Bit::new(1, 1, 1),
            Color9Bit::new(2, 1, 1),
            Color9Bit::new(3, 1, 1),
            Color9Bit::new(4, 2, 1),
            Color9Bit::new(4, 2, 1),
            Color9Bit::new(5, 2, 2),
            Color9Bit::new(5, 3, 2),
            Color9Bit::new(5, 4, 2),
            Color9Bit::new(5, 4, 3),
            Color9Bit::new(6, 4, 3),
            Color9Bit::new(6, 4, 4),
            Color9Bit::new(6, 5, 4),
            Color9Bit::new(6, 6, 4),
            Color9Bit::new(6, 6, 4),
            Color9Bit::new(6, 6, 5),
            Color9Bit::new(6, 6, 5),
        ];

        // Procedural BG Palettes. Each PaletteID matches a ColorID
        for i in 0..vid.bg_palette.len() {
            let _ = vid.push_subpalette([BG_COLOR, ColorID(i as u8), BLACK, BLACK]);
        }

        // Define new tiles
        let smiley = vid.new_tile(8, 8, &data::SMILEY);
        let checker = vid.new_tile(8, 8, &data::ARROW);
        let arrow = vid.new_tile(8, 8, &data::ARROW);

        // Set BG tiles
        for col in 0..BG_COLUMNS as u16 {
            for row in 0..BG_ROWS as u16 {
                // Calculate palette ID based on coordinates, limits to 14 indices
                let index = (col + row) % 14;
                // Adds 2 to avoid colors 0 and 1 in the BG
                let adjusted_palette = PaletteID(2 + index as u8);

                vid.bg_map.set_tile(BgBundle {
                    col,
                    row,
                    tile_id: checker,
                    flags: TileFlags::from(adjusted_palette).with_rotation(),
                });
            }
        }

        // Pre-generate smileys array
        let mut smileys: [Entity; SMILEY_COUNT] = from_fn(|i| {
            Entity {
                x: rand::random_range(0 .. BG_WIDTH as i16 - TILE_SIZE as i16) as f32,
                y: rand::random_range(0 .. BG_HEIGHT as i16 - TILE_SIZE as i16) as f32,
                tile: smiley,
                flags: PaletteID(4 + (i % 12) as u8).into(), // Avoids grayscale
            }
        });

        // Sort smileys by y position only
        smileys.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        // Store initial state and return
        Self {
            player: Entity {
                x: (vid.max_x() / 2) as f32,
                y: (vid.max_y() / 2) as f32,
                tile: arrow,
                flags: PaletteID(0).into(),
            },
            smileys,
            movement_start: 0.0,
        }
    }

    // Process the scene on each frame
    pub fn update(&mut self, vid: &mut VideoChip, app: BackendState) -> Option<SceneChange> {
        vid.start_frame();

        // ------------------------------ Input ------------------------------

        if app.pad.is_just_pressed(Button::Start) {
            vid.wrap_sprites = !vid.wrap_sprites;
            vid.wrap_bg = !vid.wrap_bg;
            // println!("Sprites wrap: {}, BG wrap: {}", vid.wrap_sprites, vid.wrap_bg);
        }

        // Increase speed if any "face" button (A,B,X,Y) is down
        let speed = if app.pad.is_any_down(AnyButton::Face) {
            30.0 * app.elapsed as f32
        } else {
            60.0 * app.elapsed as f32
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
        let target_x = (self.player.x - 16.0 - (vid.width() as f32 / 2.0)).floor() as i16;
        let target_y = (self.player.y - 16.0 - (vid.height() as f32 / 2.0)).floor() as i16;
        vid.scroll_x = target_x;
        vid.scroll_y = target_y;

        vid.color_cycle(self.player.flags.palette(), 1, 1, 15);

        for col in 0..BG_COLUMNS as u16 {
            for row in 0..BG_ROWS as u16 {
                let Some(mut flags) = vid.bg_map.get_flags(col, row) else {
                    continue;
                };
                flags.set_rotation(self.player.flags.is_rotated());
                flags.set_flip_x(self.player.flags.is_flipped_x());
                flags.set_flip_y(self.player.flags.is_flipped_y());
                vid.bg_map.set_flags(col, row, flags);
            }
        }

        // Draw shadows first (lowest priority).
        let mut sprite_shadow = |entity: &Entity| {
            vid.draw_sprite(DrawBundle {
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
            vid.draw_sprite(DrawBundle {
                x: (entity.x - 1.0).floor() as i16,
                y: (entity.y - 1.0 - hover).floor() as i16,
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

        vid.draw_sprite(DrawBundle {
            x: 0,
            y: 0,
            id: TileID(0),
            flags: TileFlags::default(),
        });

        // ------------------- Return mode switch request -------------------

        if app.pad.is_just_pressed(Button::Menu) {
            Some(SceneChange::B)
        } else {
            None
        }
    }
}
