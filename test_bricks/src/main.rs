mod gameplay;
mod init;
mod update;
mod specs;

pub use crate::{gameplay::*, specs::*};

use tato::*;
use tato_mquad::App;
use macroquad::{
    input::*,
    prelude::{Conf, next_frame},
};

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


struct Game {
    world:World<TilesetID, PaletteID>,
    atlas:Atlas<TilesetID, PaletteID>,
    paddle: Paddle,
    puck: Puck,
    bricks: EntityID,
    overlay: Vec<String>
}


#[macroquad::main(window_conf)]
async fn main() {
    let mut game = init::new_game();
    
    // Mquad App init and loop
    let mut app = App::new(&game.world);
    loop {
        app.start_frame(&mut game.world);
        
        // Update
        if is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Q) { break; }
        if is_key_pressed(KeyCode::A) { game.world.debug_atlas = !game.world.debug_atlas }
        if is_key_pressed(KeyCode::D) { game.world.debug_pivot = !game.world.debug_pivot }
        if is_key_pressed(KeyCode::C) { game.world.debug_colliders = !game.world.debug_colliders }
        if is_key_pressed(KeyCode::Escape) {
            game.puck.vel = Vec2 { x: 0.0, y: -60.0 };
            game.world.set_position(game.paddle.id, game.paddle.initial_pos);
            game.world.set_position(game.puck.id, game.puck.initial_pos);
        }

        update::move_player(&mut game);
        update::move_puck(&mut game);

        // Render
        game.world.framebuf.clear(Color24::gray_dark());
        game.world.render_frame();

        let font_left = FontInfo { tileset_id: TilesetID::Hud.into(), font: 0, depth: 255, align_right: false };
        let font_right = FontInfo { tileset_id: TilesetID::Hud.into(), font: 0, depth: 255, align_right: true };

        game.world.draw_text("1234", 8, 8, &font_left);
        game.world.draw_text("ZONE 1", 248, 8, &font_right);

        // Overlay
        app.push_overlay(format!("FPS: {:.1}", 1.0 / game.world.time_elapsed()));
        app.push_overlay(format!("Entity count: {}", game.world.entities().len()));
        app.push_overlay(format!("Update time: {:.2}", game.world.time_update() * 1000.0));
        app.push_overlay(format!("Pos: {:?}", game.world.get_position(game.paddle.id)));
        for line in &game.overlay {
            app.push_overlay(line.clone());
        }
        game.overlay.clear();

        // Finish frame
        app.finish_frame(&mut game.world);
        next_frame().await;
    }
}

