mod astro;

use crate::astro::{ASTRO_TILESET, STRIP_ASTRO};
use tato::{arena::Arena, prelude::*};
use tato_macroquad::{
    MquadBackend,
    macroquad::{
        self,
        input::is_quit_requested,
        time::get_frame_time,
        window::{Conf, next_frame},
    },
    tato_window_conf,
};

// A minimal entity that fits in a single 64 bit value! :-)
struct Entity {
    x: i16,
    y: i16,
    vel_x: i8,
    vel_y: i8,
    anim: AnimID,
    flip: bool,
}

const W: u16 = 240;
const H: u16 = 180;
const BANK_BG: u8 = 0;
const BANK_FG: u8 = 1;

#[macroquad::main(window_conf)]
async fn main() -> TatoResult<()> {
    // Init
    let mut frame_arena = Arena::<32_768, u32>::new();
    let mut tato = Tato::new(W, H, 60);
    let mut backend = MquadBackend::new(&tato).await;
    let mut dash = Dashboard::new().unwrap();
    let bg_map = Tilemap::<1024>::new(32, 32);

    tato.video.bg_color = RGBA12::new(2, 3, 4);
    tato.video.bg_tile_bank = BANK_BG;
    tato.video.fg_tile_bank = BANK_FG;

    // Animations
    let astro = tato.push_tileset(BANK_FG, ASTRO_TILESET)?;
    let strip = tato.load_animation_strip(astro, &STRIP_ASTRO)?;
    let anim_right = tato.init_anim(Anim { strip, fps: 8, rep: true, frames: [12, 13, 14, 13] })?;
    let anim_down = tato.init_anim(Anim { strip, fps: 8, rep: true, frames: [4, 5, 6, 5] })?;
    let anim_up = tato.init_anim(Anim { strip, fps: 8, rep: true, frames: [8, 9, 10, 9] })?;

    // To be able to apply delta timing to entities with integer coordinates
    // and velocities, we're using a SCALE factor - everything is scaled up
    // internally, velocities are applied, and then scaled down at the end
    // for drawing. This was a common technique in the 80's to apply simple
    // physics to 16 bit integer  coordinates.
    const SCALE: i16 = 16;

    // Entities.
    // TODO: Obtain anims from tileset, so that we can probe a frame
    // (which is just a tilemap) for its dimensions
    let sprite_w = 16;
    let sprite_h = 16;
    let min_x = 0;
    let min_y = 0;
    let max_x = (W as i16 - sprite_w) * SCALE;
    let max_y = (H as i16 - sprite_h) * SCALE;
    let mut rng = tato::rng::Rng::new(32, 123);

    let mut entities: [Entity; 20] = core::array::from_fn(|_| {
        let mut vel_x = (rng.range_i32(-1, 1) as i16 * SCALE) as i8;
        let vel_y = (rng.range_i32(-1, 1) as i16 * SCALE) as i8;
        // Velocity is never (0,0)
        if vel_x == 0 && vel_y == 0 {
            vel_x = 1;
        }
        Entity {
            x: rng.range_i32(min_x as i32, max_x as i32) as i16,
            y: rng.range_i32(min_y as i32, max_y as i32) as i16,
            vel_x,
            vel_y,
            anim: anim_right,
            flip: vel_x < 0,
        }
    });

    // Main loop
    loop {
        frame_arena.clear();
        backend.frame_start(&mut frame_arena, &mut tato.pad);
        dash.frame_start(&mut frame_arena, &mut backend);
        tato.frame_start(get_frame_time());

        for entity in &mut entities {
            // Velocity control with delta
            let vel_x = (entity.vel_x as f32 * tato.delta()) as i16;
            entity.x += vel_x as i16;
            if entity.x >= max_x || entity.x <= min_x {
                entity.vel_x *= -1;
                entity.x -= vel_x;
            }

            let vel_y = (entity.vel_y as f32 * tato.delta()) as i16;
            entity.y += vel_y as i16;
            if entity.y >= max_y || entity.y <= min_y {
                entity.vel_y *= -1;
                entity.y -= vel_y as i16;
            }

            // Anim control
            if entity.vel_x.abs() > entity.vel_y.abs() {
                entity.anim = anim_right;
                if entity.vel_x > 0 {
                    entity.flip = false;
                } else {
                    entity.flip = true;
                }
            } else {
                if entity.vel_y > 0 {
                    entity.anim = anim_down;
                } else {
                    entity.anim = anim_up;
                }
            }

            // Draw!
            tato.draw_anim(
                entity.anim,
                SpriteBundle {
                    x: entity.x / SCALE,
                    y: entity.y / SCALE,
                    flip_x: entity.flip,
                    flip_y: false,
                },
            );
        }

        tato.frame_finish();
        dash.frame_present(&mut frame_arena, &mut backend, &tato);
        backend.frame_present(&mut frame_arena, &tato, &[&bg_map]);
        if is_quit_requested() {
            break;
        }
        next_frame().await;
    }
    Ok(())
}

fn window_conf() -> Conf {
    tato_window_conf("Tato Macroquad Example", 320, 240)
}


// Curiosity! To flicker sprites that go above the sprites-per-scanline
// limit, you can do this:
// let frame_offset = tato.video.frame_number() % 2;
// for priority_group in 0..2 {
//     let actual_group = (priority_group + frame_offset) % 2;
//     for i in (0..entities.len()).filter(|&i| i % 2 == actual_group) {
//         // Draw entities in this group
//     }
// }
