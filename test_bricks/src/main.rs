mod gameplay;
mod ids;
mod input;
mod specs;
mod update;

pub use crate::{gameplay::*, ids::*, input::*, specs::*};
use macroquad::prelude::*;
use spud::{Atlas, Collider, Shape, Specs, Vec2, World};

pub type GameWorld = World<GameSpecs, TilesetID, PaletteID>;
pub type GameAtlas = Atlas<GameSpecs, TilesetID, PaletteID>;

#[macroquad::main(window_conf)]
async fn main() {
    // macroquad init
    let mut img = Image::gen_image_color(
        GameSpecs::RENDER_WIDTH as u16,
        GameSpecs::RENDER_HEIGHT as u16,
        BLACK,
    );
    let render_texture = Texture2D::from_image(&img);
    render_texture.set_filter(FilterMode::Nearest);

    // spud init
    let atlas: GameAtlas = Atlas::load(include_bytes!("../assets/converted/atlas"));
    let mut world: GameWorld = World::new();
    world.debug_colliders = true;
    // world.debug_pivot = true;   
    world.render.load_palettes_from_atlas(&atlas);
    world.render.push_tileset(&atlas, TilesetID::Hud);
    world.render.push_tileset(&atlas, TilesetID::Bg);
    world.render.push_tileset(&atlas, TilesetID::Sprites);

    let bg = world.add_entity(0);
    world.set_shape(bg, Shape::Bg {
        tileset: TilesetID::Bg.into(),
        tilemap_id: 0,
    });
    world.set_collider(bg, Collider::new_tilemap_collider());

    let mut paddle = Paddle {
        id: {
            let paddle = world.add_entity(0);
            world.set_shape(paddle, Shape::sprite_from_anim(TilesetID::Sprites, 0));
            world.set_collider(paddle, Collider::from(spud::Rect{x:-12.0, y:-8.0, w:24.0, h:16.0}));
            world.set_position(paddle, 128.0, 168.0);
            world.set_render_offset(paddle, -12,-8);
            paddle
        },
        input: Input::default(),
        vel: spud::Vec2::default(),
    };

    let initial_pos = spud::Vec2 { x: 128.0, y: 124.0 };
    let mut puck = Puck {
        id: {
            let puck = world.add_entity(0);
            world.set_shape(puck, Shape::sprite_from_anim(TilesetID::Sprites, 1));
            world.set_position(puck, initial_pos.x, initial_pos.y);
            world.set_collider(puck, Collider::from(Vec2::zero()));
            world.set_render_offset(puck, -3, -4 );
            puck
        },
        initial_pos,
        vel: Vec2 { x: 60.0, y: 0.0 },
    };

    let time = std::time::Instant::now();
    // main loop (infinite until "break")
    loop {
        // Update
        world.start_frame(time.elapsed().as_secs_f32());

        paddle.input = Input::default();
        if is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Q) {
            break;
        }
        if is_key_down(KeyCode::Up) {
            paddle.input.up = true
        }
        if is_key_down(KeyCode::Down) {
            paddle.input.down = true
        }
        if is_key_down(KeyCode::Left) {
            paddle.input.left = true
        }
        if is_key_down(KeyCode::Right) {
            paddle.input.right = true
        }
        if is_key_pressed(KeyCode::A) {
            world.debug_atlas = !world.debug_atlas
        }
        if is_key_pressed(KeyCode::D) {
            world.debug_pivot = !world.debug_pivot
        }
        if is_key_pressed(KeyCode::Escape) {
            if let Some(ent) = world.get_entity_mut(puck.id) {
                ent.pos = puck.initial_pos;
                puck.vel = Vec2 { x: 60.0, y: 30.0 };
            }
        }

        world.use_collider(bg, Vec2::zero());
        update::move_player(&mut paddle, &mut world);
        update::move_puck(&mut puck, &mut world);

        // world.resolve_collisions();

        // Render
        world.render_frame();
        world.draw_text("1234", 8, 8, TilesetID::Hud, 0, false);
        world.draw_text("ZONE 1", 248, 8, TilesetID::Hud, 0, true);

        // Copy from framebuffer to macroquad texture
        let source = world.framebuf.pixels();
        let width = GameSpecs::RENDER_WIDTH;
        for y in 0..GameSpecs::RENDER_HEIGHT {
            for x in 0..GameSpecs::RENDER_WIDTH {
                let source_index = (y * width) + x;
                let color = source[source_index];
                img.set_pixel(
                    x as u32,
                    y as u32,
                    Color::from_rgba(color.r, color.g, color.b, color.a),
                )
            }
        }

        // Render texture to screen
        clear_background(BLACK);
        let scale = (screen_height() / GameSpecs::RENDER_HEIGHT as f32).floor();
        let render_width = GameSpecs::RENDER_WIDTH as f32 * scale;
        let render_height = GameSpecs::RENDER_HEIGHT as f32 * scale;
        let x = (screen_width() - render_width) / 2.0;
        let y = (screen_height() - render_height) / 2.0;

        render_texture.update(&img);
        draw_texture_ex(
            &render_texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(render_width, render_height)),
                source: None,
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );

        // Finish (calculate timings)
        world.finish_frame(time.elapsed().as_secs_f32());
        next_frame().await;
    }
}

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
