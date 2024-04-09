
mod update;
mod actors; 
mod specs;
mod init;
mod grid;

pub use crate::{
    actors::*,
    specs::*,
    grid::*
};

use macroquad::prelude as mquad;
use tato_mquad::App;
use tato::*;


pub struct Game {
    world:World<TilesetID, PaletteID>,
    player: Player,
    stars_bg_0:EntityID,
    stars_bg_1:EntityID,
    stars_fg_0:EntityID,
    stars_fg_1:EntityID,
    bullets:RingPool<EntityID, 16>,
    enemies: Grid,
    // cooldown:f32,
}


fn window_conf() -> mquad::Conf {
    mquad::Conf {
        window_title: "Bug's Revenge".into(),
        fullscreen: false,
        high_dpi: true,
        sample_count: 0,
        window_resizable: true,
        window_width: 256 * 3,
        window_height: 192 * 3,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Init 
    let mut game = init::new_game(SPECS);
    let mut app = App::new(&game.world);
    
    // Main loop
    loop {
        app.start_frame(&mut game.world);
                
        // Update game and render entities 
        if mquad::is_key_down(mquad::KeyCode::LeftSuper) && mquad::is_key_pressed(mquad::KeyCode::Q) { break; }

        update::frame(&mut game);
        game.world.framebuf.clear(Color24::black());
        game.world.render_frame();

        // Overlay
        app.push_overlay(format!("FPS: {:.1}", 1.0 / game.world.time_elapsed()));
        app.push_overlay(format!("Entity count: {}", game.world.entities().len()));
        app.push_overlay(format!("Update time: {:.2}", game.world.time_update() * 1000.0));
        for bullet in game.bullets.iter() {
            app.push_overlay(format!("{:?}", bullet));
        }

        // Finish frame
        app.finish_frame(&mut game.world);
        mquad::next_frame().await;
    }
}


