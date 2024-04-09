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
        window_title: "Paddlenoid".into(),
        fullscreen: false,
        high_dpi: true,
        sample_count: 0,
        window_resizable: true,
        window_width: 288 * 3,
        window_height: 216 * 3,
        ..Default::default()
    }
}

enum State {
    None,
    Title(Title),
    Game(Game)
}

enum NewState {
    None,
    Title,
    Game
}

struct Title {
    bg: EntityID
}

struct Game {
    // world:World<TilesetID, PaletteID>,
    paddle: Paddle,
    puck: Puck,
    bricks: EntityID,
    overlay: Vec<String>
}

pub type Atlas = tato::Atlas<TilesetID, PaletteID>;
pub type World = tato::World<TilesetID, PaletteID>;

#[macroquad::main(window_conf)]
async fn main() {
    let atlas = Atlas::load(SPECS, include_bytes!("../assets/converted/atlas"));
    let font_left = FontInfo { tileset_id: TilesetID::Hud.into(), font: 0, depth: 200, align_right: false };
    let font_right = FontInfo { tileset_id: TilesetID::Hud.into(), font: 0, depth: 200, align_right: true };

    let mut new_state = NewState::Title;
    let mut state = State::None;
    let mut world = World::new(SPECS);

    // Mquad App init and loop
    let mut app = App::new(&world);
    loop {
        // Init state if new_state changed.
        match new_state {
            NewState::None => {},
            NewState::Title => {
                world.reset();
                state = init::title(&atlas, &mut world);
            },
            NewState::Game => {
                world.reset();
                state = init::game(&atlas, &mut world);
            },
        }

        // new_state is always set to None on every frame after processing it.
        // The update code below can set it to something else as needed.
        new_state = NewState::None;

        // Frame start
        app.start_frame(&mut world);

        // Update for all states
        if is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Q) { break; }
        if is_key_pressed(KeyCode::A) { world.debug_atlas = !world.debug_atlas }
        if is_key_pressed(KeyCode::D) { world.debug_pivot = !world.debug_pivot }
        if is_key_pressed(KeyCode::C) { world.debug_colliders = !world.debug_colliders }
        if is_key_pressed(KeyCode::O) { app.display_overlay = ! app.display_overlay }

        // State specific update
        match state {
            State::None => {}
            State::Title(_) => {
                world.framebuf.clear(Color24::black());
                world.draw_text("PRESS START", 80, 160, &font_left);
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Z) {
                    new_state = NewState::Game;
                }
            },
            State::Game(ref mut game) => {
                world.framebuf.clear(Color24::gray_dark());
                if is_key_pressed(KeyCode::Escape) {
                    new_state = NewState::Title;
                }

                update::move_player(game, &mut world);
                update::move_puck(game, &mut world);

                world.draw_text("1234", 8, 8, &font_left);
                world.draw_text("ZONE 1", 248, 8, &font_right);

                app.push_overlay(format!("Pos: {:?}", world.get_position(game.paddle.id)));
                // Additional  game overlay
                (0 .. game.overlay.len()).rev().for_each(|_|{
                    if let Some(line) = game.overlay.pop(){
                        app.push_overlay(line);
                    }
                });
        
            }
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


