mod astro;

use crate::astro::{ASTRO_TILESET, STRIP_ASTRO};
use tato::prelude::*;
use tato_raylib::RaylibBackend;

// An entity that fits in 64 bits! :-)
struct Entity {
    x: i16,
    y: i16,
    vel_x: i8,
    vel_y: i8,
    anim: u8,
    flip: bool,
}

const W: u16 = 240;
const H: u16 = 180;
const BANK_BG: u8 = 0;
const BANK_FG: u8 = 1;

fn main() {
    // Init
    let bg_map = Tilemap::<1024>::new(32, 32);
    let mut tato = Tato::new(W, H, 60);
    let mut backend = RaylibBackend::new(&tato);

    tato.video.bg_color = RGBA12::new(2, 3, 4, 7);
    tato.video.bg_tile_bank = BANK_BG;
    tato.video.fg_tile_bank = BANK_FG;

    // Animations
    // TODO: These operations should return a Result with a TatoError.
    let astro = tato.push_tileset(BANK_FG, ASTRO_TILESET).unwrap();
    let strip_astro = tato.load_animation_strip(astro, &STRIP_ASTRO).unwrap();
    let anims = [
        Anim { fps: 8, repeat: true, frames: [12, 13, 14, 13] }, // right
        Anim { fps: 8, repeat: true, frames: [4, 5, 6, 5] },     // down
        Anim { fps: 8, repeat: true, frames: [8, 9, 10, 9] },    // up
    ];

    // Entities.
    // TODO: Obtain anims from tileset, so that we can probe a frame
    // (which is just a tilemap) for its dimensions
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
            anim: 0,
            flip: vel_x < 0,
        }
    });

    // Main loop
    while !backend.ray.window_should_close() {
        tato.video.start_frame();
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

            // Anim control
            if entity.vel_x.abs() > entity.vel_y.abs() {
                entity.anim = 0; // RIGHT
                if entity.vel_x > 0 {
                    entity.flip = false;
                } else {
                    entity.flip = true;
                }
            } else {
                if entity.vel_y > 0 {
                    entity.anim = 1 // DOWN;
                } else {
                    entity.anim = 2; //UP
                }
            }

            // Draw!
            tato.draw_anim(
                strip_astro,
                &anims[entity.anim as usize],
                SpriteBundle { x: entity.x, y: entity.y, flip_x: entity.flip, flip_y: false },
            );
        }
        backend.render(&tato, &[&bg_map]);
    }
}

// Curiosity! To flicker sprites that go above the sprites-per-scanline
// limit, you can do this:
// let frame_offset = tato.video.frame_count() % 2;
// for priority_group in 0..2 {
//     let actual_group = (priority_group + frame_offset) % 2;
//     for i in (0..entities.len()).filter(|&i| i % 2 == actual_group) {
//         // Draw code
//     }
// }
