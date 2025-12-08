use crate::*;
use tato::default_assets::*;
use tato::prelude::*;

#[derive(Debug)]
pub struct SceneB {
    player: Entity,
    smileys: [Entity; 64],
}

impl SceneB {
    pub fn new(t: &mut Tato, state: &mut State) -> TatoResult<Self> {
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
        t.banks[0].reset();
        t.banks[0].load_default_colors();

        t.banks[1].reset();
        t.banks[1].load_default_colors();

        t.video.bg_color = RGBA12::BLACK;
        t.video.crop_color = RGBA12::DARK_GREEN;

        let _tileset = t.push_tileset(0, DEFAULT_TILESET)?;
        let tile = TILE_SMILEY;

        // Define color mappings
        {
            let mut mapping:[u8; COLORS_PER_PALETTE as usize] = Default::default();
            mapping[2] = 8;
            t.banks[0].color_mapping[2] = mapping;
        }
        for n in 0 .. 3 {
            let mut mapping:[u8; COLORS_PER_PALETTE as usize] = Default::default();
            mapping[2] = 9 + n;
            t.banks[1].push_color_mapping(mapping);
        }

        // Set BG cells to use mappings
        for (i, cell) in state.bg.cells.iter_mut().enumerate() {
            cell.id = tile;
            cell.flags = TileFlags::default();
            cell.color_mapping = (i % 3) as u8 + 1;
        }

        Ok(Self {
            player: Entity {
                x: x as f32,
                y: y as f32,
                tile,
                flags: TileFlags::default(),
                color_mapping: MAP_CYCLE,
            },
            smileys: core::array::from_fn(|_| Entity {
                // Will test wrapping of large f32 value into i16
                // using "wrap_width" and "wrap_height"
                x: rand::random_range(0.0..255.0), // - 32_000.0,
                y: rand::random_range(0.0..255.0), // + 32_000.0,
                tile,
                flags: TileFlags::default(),
                color_mapping: 2,
            }),
        })
    }

    pub fn update(&mut self, t: &mut Tato, state: &mut State) -> Option<SceneChange> {
        let speed = 1.0;

        // Input
        if state.pad.is_down(Button::Left) {
            self.player.x -= speed;
        } else if state.pad.is_down(Button::Right) {
            self.player.x += speed;
        }
        if state.pad.is_down(Button::Up) {
            self.player.y -= speed;
        } else if state.pad.is_down(Button::Down) {
            self.player.y += speed;
        }

        // Draw!
        {
            let cycle_color = &mut t.banks[0].color_mapping[MAP_CYCLE as usize][2];
            *cycle_color = ((*cycle_color + 1) % 12) + 4;
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
                color_mapping: entity.color_mapping,
            });
        }

        t.video.draw_fg_tile(DrawBundle {
            x: self.player.x as i16,
            y: self.player.y as i16,
            id: self.player.tile,
            flags: self.player.flags,
            color_mapping: MAP_CYCLE,
        });

        if state.pad.is_just_pressed(Button::Start) {
            t.video.wrap_sprites = !t.video.wrap_sprites;
        }

        if state.pad.is_just_pressed(Button::Menu) { Some(SceneChange::C) } else { None }
    }
}
