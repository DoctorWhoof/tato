mod gameplay;
mod init;
mod update;
mod specs;

use tato::*;
use tato_mquad::App;
use macroquad::{input::*, window::*};

pub use crate::{gameplay::*, specs::*};

fn window_conf() -> Conf {
    Conf {
        window_title: "Ponck".into(),
        fullscreen: false,
        high_dpi: true,
        sample_count: 0,
        window_resizable: true,
        window_width: 288 * 3,
        window_height: 216 * 3,
        ..Default::default()
    }
}

enum Scene {
    None,
    Title,
    Gameplay(Option<Game>) // The Game struct is optional since it can be not-initialized
}


struct Game {
    // world:World<TilesetID, PaletteID>,
    paddle: Paddle,
    puck: Puck,
    bricks: EntityID,
    score: u32,
    zone: u8,
    zone_count: u8,
    overlay: Vec<String>
}

pub type Atlas = tato::Atlas<TilesetID, PaletteID>;
pub type World = tato::World<TilesetID, PaletteID>;

#[macroquad::main(window_conf)]
async fn main() {
    let mut world = World::new(SPECS);
    let atlas = Atlas::load(SPECS, include_bytes!("../assets/converted/atlas"));
    let font_left = FontInfo { tileset_id: TilesetID::Hud.into(), font: 0, depth: 200, align_right: false };
    let font_right = FontInfo { tileset_id: TilesetID::Hud.into(), font: 0, depth: 200, align_right: true };

    let mut new_state = Scene::Gameplay(None);
    let mut state = Scene::None;

    // Mquad App init and loop
    let mut app = App::new(&world);
    loop {
        // Init state if new_state changed.
        match new_state {
            Scene::None => {},
            Scene::Title => {
                world.reset();
                state = init::title(&atlas, &mut world);
            },
            Scene::Gameplay(_) => {
                world.reset();
                state = init::game(&atlas, &mut world);
            },
        }

        // new_state is always set to None on every frame after processing it.
        // The update code below can set it to something else as needed.
        new_state = Scene::None;

        // Frame start
        app.start_frame(&mut world);

        // Update for all states
        if is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Q) { break; }
        if is_key_pressed(KeyCode::A) { world.debug_atlas = !world.debug_atlas }
        if is_key_pressed(KeyCode::D) { world.debug_wireframe = !world.debug_wireframe }
        if is_key_pressed(KeyCode::C) { world.debug_colliders = !world.debug_colliders }
        if is_key_pressed(KeyCode::O) { app.display_overlay = ! app.display_overlay }

        // Scene specific update
        match state {
            Scene::Title => {
                world.framebuf.clear(Color24::gray());
                world.draw_text("PRESS START", 80, 160, &font_left);
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Z) {
                    new_state = Scene::Gameplay(None);
                }
            },
            Scene::Gameplay(Some(ref mut game)) => {
                world.framebuf.clear(Color24::gray_dark());
                if is_key_pressed(KeyCode::Escape) {
                    new_state = Scene::Title;
                }

                update::move_player(game, &mut world);
                update::move_puck(game, &mut world);

                world.draw_text(&format!("{}", game.score), 8, 8, &font_left);
                world.draw_text("ZONE 1", 248, 8, &font_right);

                app.push_overlay(format!("Pos: {:?}", world.get_position(game.paddle.id)));
                // Additional  game overlay
                (0 .. game.overlay.len()).rev().for_each(|_|{
                    if let Some(line) = game.overlay.pop(){
                        app.push_overlay(line);
                    }
                });
        
            },
            _ => {}
        }
        
        // Render
        world.render_frame();

        // Overlay
        app.push_overlay(format!("FPS: {:.1}", world.fps()));
        app.push_overlay(format!("Entity count: {}", world.entities().len()));
        app.push_overlay(format!("Update time: {:.2}", world.time_update() * 1000.0));

        // Finish frame
        app.finish_frame(&mut world);
        next_frame().await;
    }
}


