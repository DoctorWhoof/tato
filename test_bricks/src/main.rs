mod gameplay;
mod input;
mod update;
mod specs;

pub use crate::{gameplay::*, input::*, specs::*};

use tato_mquad::App;
use macroquad::prelude::*;
use tato::{Atlas, Collider, Color24, Shape, Vec2, World};

fn window_conf() -> Conf {
    Conf {
        window_title: "Paddlenoid".into(),
        fullscreen: false,
        high_dpi: true,
        sample_count: 0,
        window_resizable: true,
        window_width: (216.0 * 1.79) as i32 * 3,
        window_height: 216 * 3,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // spud init
    let mut world = World::new(SPECS);
    let atlas = Atlas::<TilesetID, PaletteID>::load(SPECS, include_bytes!("../assets/converted/atlas"));

    world.debug_colliders = true;
    world.renderer.load_palettes_from_atlas(&atlas);
    world.renderer.load_tileset(&atlas, TilesetID::Hud);
    world.renderer.load_tileset(&atlas, TilesetID::Bg);
    world.renderer.load_tileset(&atlas, TilesetID::Sprites);

    // Game init
    let bg = world.add_entity(0);
    world.set_shape(bg, Shape::Bg {
        tileset: TilesetID::Bg.into(),
        tilemap_id: 0,
    });
    world.add_collider(bg, Collider::new_tilemap_collider());

    let initial_paddle_pos = tato::Vec2 { x: 128.0, y: 160.0 };
    let mut paddle = Paddle {
        id: {
            let paddle = world.add_entity(0);
            world.set_shape(paddle, Shape::sprite_from_anim(TilesetID::Sprites, 0));
            world.add_collider(paddle, Collider::from(tato::Rect{x:-11.0, y:-7.0, w:22.0, h:14.0}));
            world.set_position(paddle, initial_paddle_pos);
            world.set_render_offset(paddle, -12,-8);
            paddle
        },
        input: Input::default(),
        vel: tato::Vec2::default(),
    };

    let initial_puck_pos = tato::Vec2 { x: 128.0, y: 124.0 };
    let mut puck = Puck {
        id: {
            let puck = world.add_entity(0);
            world.set_shape(puck, Shape::sprite_from_anim(TilesetID::Sprites, 1));
            world.set_position(puck, initial_puck_pos);
            world.add_collider(puck, Collider::from(Vec2::zero()));
            world.set_render_offset(puck, -3, -4 );
            puck
        },
        initial_pos: initial_puck_pos,
        vel: Vec2 { x: 0.0, y: 0.0 },
    };
    
    // Mquad App init and loop
    let mut app = App::new(&world);
    loop {
        app.start_frame(&mut world);
        
        // Update
        paddle.input = Input::default();
        if is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Q) { break; }
        if is_key_down(KeyCode::Up) { paddle.input.up = true }
        if is_key_down(KeyCode::Down) { paddle.input.down = true }
        if is_key_down(KeyCode::Left) { paddle.input.left = true }
        if is_key_down(KeyCode::Right) { paddle.input.right = true }
        if is_key_pressed(KeyCode::A) { world.debug_atlas = !world.debug_atlas }
        if is_key_pressed(KeyCode::D) { world.debug_pivot = !world.debug_pivot }
        if is_key_pressed(KeyCode::Escape) {
            puck.vel = Vec2 { x: 0.0, y: -60.0 };
            world.set_position(paddle.id, initial_paddle_pos);
            world.set_position(puck.id, initial_puck_pos);
        }

        world.use_static_collider(bg);

        update::move_player(&mut paddle, &mut world);
        update::move_puck(&mut puck, &mut world);

        // Render
        world.framebuf.clear(Color24::gray_dark());
        world.render_frame();
        world.draw_text("1234", 8, 8, TilesetID::Hud, 0, false, 255);
        world.draw_text("ZONE 1", 248, 8, TilesetID::Hud, 0, true, 255);

        // Overlay
        app.push_overlay(format!("FPS: {:.1}", 1.0 / world.time_elapsed()));
        app.push_overlay(format!("Entity count: {}", world.entities().len()));
        app.push_overlay(format!("Update time: {:.2}", world.time_update() * 1000.0));

        // Finish frame
        app.finish_frame(&mut world);
        next_frame().await;
    }
}

