use crate::*;
use core::array::from_fn;
use tato::default_assets::*;
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
    pub fn new(t: &mut Tato, banks:&mut [Bank], state: &mut State) -> TatoResult<Self> {
        t.video.reset_all();
        t.video.bg_tile_bank = 1; // uses bank 1 for BG tiles
        t.video.bg_color = RGBA12::new(2, 1, 0);
        t.video.wrap_bg = false;
        t.video.wrap_sprites = false;

        // Line scrolling effect, adjusts on every line
        t.video.irq_line = Some(|iter, video, _bg| {
            let line_offset = (iter.y() as f32 + video.scroll_y as f32) / 16.0;
            let phase = ((video.frame_number() as f32 / 30.0) + line_offset).sin();
            iter.scroll_x = (video.scroll_x as f32 - (phase * 8.0)) as i16;
        });

        banks[0].reset();
        banks[0].load_default_colors();
        banks[0].append_tiles(&BANK_DEFAULT, None).unwrap();
        // Palette test - defines BG palette with a golden tint!
        banks[1].reset();
        banks[1].append(&BANK_DEFAULT).unwrap();
        banks[1].palette = [
            RGBA12::TRANSPARENT,
            RGBA12::new(2, 1, 1),
            RGBA12::new(3, 1, 1),
            RGBA12::new(4, 2, 1),
            RGBA12::new(4, 2, 1),
            RGBA12::new(5, 2, 2),
            RGBA12::new(5, 3, 2),
            RGBA12::new(5, 4, 2),
            RGBA12::new(5, 4, 3),
            RGBA12::new(6, 4, 3),
            RGBA12::new(6, 4, 4),
            RGBA12::new(6, 5, 4),
            RGBA12::new(6, 6, 4),
            RGBA12::new(6, 6, 5),
            RGBA12::new(7, 6, 5),
            RGBA12::new(7, 7, 5),
        ];

        for (i, mapping) in banks[0].color_mapping[COLORMAP_CYCLE as usize].iter().enumerate() {
            let color = banks[0].palette[*mapping as usize];
            println!("{}: {} ({})", i, mapping, color);
        }

        // Color mappings.
        {
            // mapping 0 is for the shadow
            let mapping = &mut banks[0].color_mapping[COLORMAP_SHADOW as usize];
            mapping[0] = 0; // 0 stays transparent
            mapping[1] = 1;
            mapping[2] = 1;
            mapping[3] = 1
        }

        {
            // mappings 2 to 15 replace color 2 with a color from the palette
            for i in 2..COLORS_PER_PALETTE as usize {
                let mapping = &mut banks[0].color_mapping[i];
                mapping[2] = (i as u8 % 12) + 4;
            }
        }

        {
            // And now the BG palette
            for i in 0..COLORS_PER_PALETTE as usize {
                let bg_color = (i % 16) as u8;
                let mapping = &mut banks[1].color_mapping[i];
                mapping[0] = 0; // 0 stays transparent
                mapping[1] = bg_color;
                mapping[2] = bg_color;
                mapping[3] = bg_color;
            }
        }

        // Define new tiles
        // let _tileset_fg = t.push_tileset(0, DEFAULT_TILESET)?;
        // let _tileset_bg = t.push_tileset(1, DEFAULT_TILESET)?;

        // Set BG tiles, acquire width and height of bg map
        let (w, h) = {
            for col in 0..state.bg.columns() as i16 {
                for row in 0..state.bg.rows() as i16 {
                    state.bg.set_op(BgOp {
                        col,
                        row,
                        tile_id: TILE_ARROW.id,
                        flags: TileFlags::default().with_rotation(),
                        // Calculate palette ID based on coordinates, limits to 14
                        // indices, adds 2 to avoid colors 0 and 1 in the BG
                        color_mapping: ((col + row) % 14) as u8 + 2,
                    });
                }
            }
            (state.bg.width() as i16, state.bg.height() as i16)
        };

        // Pre-generate smileys array
        let mut smileys: [Entity; SMILEY_COUNT] = from_fn(|i| Entity {
            x: rand::random_range(0..w - TILE_SIZE as i16) as f32,
            y: rand::random_range(0..h - TILE_SIZE as i16) as f32,
            tile: TILE_SMILEY.id,
            flags: TileFlags::default(),
            color_mapping: (i as u8 % 14) + 2,
        });
        // Sort smileys by y position only
        smileys.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        // Store initial state and return
        Ok(Self {
            player: Entity {
                x: (t.video.width() / 2) as f32,
                y: (t.video.height() / 2) as f32,
                tile: TILE_ARROW.id,
                flags: TileFlags::default(),
                color_mapping: COLORMAP_CYCLE,
            },
            smileys,
            movement_start: 0.0,
        })
    }

    // Process the scene on each frame
    pub fn update(&mut self, t: &mut Tato, banks:&mut [Bank], state: &mut State) -> Option<SceneChange> {

        // ------------------------------ Input ------------------------------

        if t.pad.is_just_pressed(Button::Start) {
            t.video.wrap_sprites = !t.video.wrap_sprites;
            t.video.wrap_bg = !t.video.wrap_bg;
        }

        if t.paused { return None }

        // Increase speed if any "face" button (A,B,X,Y) is down
        // let speed = if t.pad.is_any_down(AnyButton::Face) {
        let speed = if t.pad.is_down(Button::A) {
            30.0 * state.elapsed as f32
        } else {
            120.0 * state.elapsed as f32
        };

        // Ensures animation always starts from phase = 0.0;
        if t.pad.is_any_just_pressed(AnyButton::Direction) {
            self.movement_start = state.time as f32;
        }

        // Player Movement
        let is_walking = {
            let (mut vel_x, mut vel_y) = (0.0, 0.0);
            if t.pad.is_down(Button::Left) {
                vel_x = -speed;
                self.player.flags.rotate_left();
            } else if t.pad.is_down(Button::Right) {
                vel_x = speed;
                self.player.flags.rotate_right();
            }
            if t.pad.is_down(Button::Up) {
                vel_y = -speed;
                self.player.flags.rotate_up();
            } else if t.pad.is_down(Button::Down) {
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

        {
            let cycle_color = &mut banks[0].color_mapping[COLORMAP_CYCLE as usize][2];
            *cycle_color = ((*cycle_color + 1) % 12) + 4;
        }

        for col in 0..state.bg.columns() as i16 {
            for row in 0..state.bg.rows() as i16 {
                let Some(mut flags) = state.bg.get_flags(col, row) else {
                    continue;
                };
                flags.set_rotation(self.player.flags.is_rotated());
                flags.set_flip_x(self.player.flags.is_flipped_x());
                flags.set_flip_y(self.player.flags.is_flipped_y());
                state.bg.set_flags(col, row, flags);
            }
        }

        // Draw shadows first (lowest priority).
        let mut sprite_shadow = |entity: &Entity| {
            t.video.draw_fg_tile(DrawBundle {
                x: entity.x as i16,
                y: entity.y as i16,
                id: entity.tile,
                flags: entity.flags,
                color_mapping: COLORMAP_SHADOW,
            });
        };
        for entity in &self.smileys {
            sprite_shadow(entity);
        }
        sprite_shadow(&self.player);

        // Draw Sprites with hover animation
        let mut sprite_hover = |entity: &Entity, phase: f32, speed: f32, height: f32| {
            let hover = ((phase * speed).sin() + 1.0) * height;
            t.video.draw_fg_tile(DrawBundle {
                x: (entity.x - 1.0) as i16,
                y: (entity.y - 1.0 - hover) as i16,
                id: entity.tile,
                flags: entity.flags,
                color_mapping: entity.color_mapping,
            });
        };

        for entity in &self.smileys {
            // passing x as phase gives us out-of-sync motion
            let hover_phase = entity.x + state.time as f32;
            sprite_hover(entity, hover_phase, 6.0, 1.5);
        }

        // Player goes in front. Drawing a sprite last means it has highest priority!
        let hover_phase = state.time as f32 - self.movement_start;
        let hover_speed = if is_walking { 24.0 } else { 4.0 };
        let hover_height = if is_walking { 2.0 } else { 1.5 };
        sprite_hover(&self.player, hover_phase, hover_speed, hover_height);

        // Flashing Smiley at the origin
        t.video.draw_fg_tile(DrawBundle {
            x: 0,
            y: 0,
            id: TILE_SMILEY.id,
            flags: TileFlags::default(),
            color_mapping: COLORMAP_CYCLE,
        });

        // ------------------- Return mode switch request -------------------

        if t.pad.is_just_pressed(Button::Menu) {
            println!("Menu");
            Some(SceneChange::B)
        } else {
            None
        }
    }
}
