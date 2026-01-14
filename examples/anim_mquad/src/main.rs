mod astro;

use crate::astro::{BANK_ASTRO, STRIP_ASTRO};
use tato::{
    arena::{Arena, ArenaOps},
    prelude::*,
};
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
    anim_frames: [u8; 4], // Frame sequence for this entity's animation
    anim_fps: u8,
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

    // Animation frame sequences (indices into STRIP_ASTRO)
    let anim_right_frames = [12, 13, 14, 13];
    let anim_down_frames = [4, 5, 6, 5];
    let anim_up_frames = [8, 9, 10, 9];

    // To be able to apply delta timing to entities with integer coordinates
    // and velocities, we're using a SCALE factor - everything is scaled up
    // internally, velocities are applied, and then scaled down at the end
    // for drawing. This was a common technique in the 80's to apply simple
    // physics to 16 bit integer coordinates.
    const SCALE: i16 = 16;

    // Entities - sprite info
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
            anim_frames: anim_right_frames,
            anim_fps: 8,
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

            // Anim control - choose which animation based on direction
            if entity.vel_x.abs() > entity.vel_y.abs() {
                entity.anim_frames = anim_right_frames;
                entity.flip = entity.vel_x < 0;
            } else {
                if entity.vel_y > 0 {
                    entity.anim_frames = anim_down_frames;
                } else {
                    entity.anim_frames = anim_up_frames;
                }
                entity.flip = false;
            }

            // Calculate current frame in animation
            let frame_idx =
                anim_get_frame(tato.video.frame_number, &entity.anim_frames, entity.anim_fps, true);
            let strip_frame = entity.anim_frames[frame_idx] as usize;

            // Draw the sprite using the tilemap from the const strip
            if let Some(tilemap) = STRIP_ASTRO.get(strip_frame) {
                tato.draw_tilemap_to_fg(
                    tilemap,
                    SpriteBundle {
                        x: entity.x / SCALE,
                        y: entity.y / SCALE,
                        flip_x: entity.flip,
                        flip_y: false,
                    },
                );
            }
        }

        tato.frame_finish();
        dash.frame_present(&mut frame_arena, &[BANK_ASTRO], &tato, &mut backend);
        backend.frame_present(&mut frame_arena, &tato, &[BANK_ASTRO], &[&bg_map]);
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
