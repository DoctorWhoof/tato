mod astro;

use crate::astro::*;
use tato::{
    arena::{Arena, ArenaOps},
    prelude::*,
};
use tato_macroquad::{
    MquadBackend,
    macroquad::{
        self,
        window::{Conf, next_frame},
    },
    tato_window_conf,
};

// A minimal entity
struct Entity {
    x: i16,
    y: i16,
    vel_x: i8,
    vel_y: i8,
    flip: bool,
    anim: &'static Anim<'static, 4>,
}

const W: u16 = 240;
const H: u16 = 180;
const BANK_BG: u8 = 0;
const BANK_FG: u8 = 1;
const ARENA_LEN: usize = 64 * 1024;

#[macroquad::main(window_conf)]
async fn main() -> TatoResult<()> {
    // Init
    let mut frame_arena = Arena::<ARENA_LEN, u32>::new();
    let mut tato = Tato::new(W, H, 60);
    let mut backend = MquadBackend::new(&tato).await;
    let mut dash = Dashboard::new().unwrap();
    let bg_map = Tilemap::<1024>::new(32, 32);

    tato.video.bg_color = RGBA12::new(2, 3, 4);
    tato.video.bg_tile_bank = BANK_BG;
    tato.video.fg_tile_bank = BANK_FG;

    // Entities - sprite info
    let sprite_w = 16;
    let sprite_h = 16;
    let min_x = 0;
    let min_y = 0;
    let max_x = W as i16 - sprite_w;
    let max_y = H as i16 - sprite_h;
    let mut rng = tato::rng::Rng::new(32, 123);

    let mut entities: [Entity; 20] = core::array::from_fn(|_| {
        let mut vel_x = rng.range_i32(-1, 1) as i8;
        let vel_y = rng.range_i32(-1, 1) as i8;
        // Velocity is never (0,0)
        if vel_x == 0 && vel_y == 0 {
            vel_x = 1;
        }
        Entity {
            x: rng.range_i32(min_x as i32, max_x as i32) as i16,
            y: rng.range_i32(min_y as i32, max_y as i32) as i16,
            vel_x,
            vel_y,
            anim: &ANIM_RIGHT,
            flip: vel_x < 0,
        }
    });

    // Main loop
    while !backend.should_close() {
        frame_arena.clear();
        backend.frame_start(&mut frame_arena, &mut tato.pad);
        dash.frame_start(&mut frame_arena, &mut backend);
        tato.frame_start(1.0 / 60.0);

        for entity in &mut entities {
            // Velocity control
            entity.x += entity.vel_x as i16;
            if entity.x >= max_x || entity.x <= min_x {
                entity.vel_x *= -1;
                entity.x += entity.vel_x as i16;
            }
            entity.y += entity.vel_y as i16;
            if entity.y >= max_y || entity.y <= min_y {
                entity.vel_y *= -1;
                entity.y += entity.vel_y as i16;
            }

            // Anim control - choose which animation based on direction
            if entity.vel_x.abs() > entity.vel_y.abs() {
                entity.anim = &ANIM_RIGHT;
                entity.flip = entity.vel_x < 0;
            } else {
                if entity.vel_y > 0 {
                    entity.anim = &ANIM_DOWN;
                } else {
                    entity.anim = &ANIM_UP;
                }
                entity.flip = false;
            }

            // Calculate current frame in animation
            let frame_idx = anim_get_frame(tato.video.frame_number, entity.anim);
            let strip_frame = entity.anim.frames[frame_idx] as usize;

            // Draw the sprite using the tilemap from the const strip
            if let Some(tilemap) = STRIP_ASTRO.get(strip_frame) {
                tato.draw_tilemap_to_fg(
                    tilemap,
                    SpriteBundle { x: entity.x, y: entity.y, flip_x: entity.flip, flip_y: false },
                );
            }
        }

        tato.frame_finish();
        dash.frame_present(&mut frame_arena, &[BANK_ASTRO], &tato, &mut backend);
        backend.frame_present(&mut frame_arena, &tato, &[BANK_ASTRO], &[&bg_map]);
        next_frame().await;
    }
    Ok(())
}

fn window_conf() -> Conf {
    tato_window_conf("Tato Macroquad Example", 320, 240)
}
