mod specs;
pub use crate::specs::*;

use macroquad::prelude as mquad;
use tato_mquad::App;
use tato::*;


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
    let atlas = Atlas::<TilesetID, PaletteID>::load(SPECS, include_bytes!("../assets/converted/atlas"));
    let mut world = World::<TilesetID, PaletteID>::new(SPECS);
    world.renderer.load_tileset(&atlas, TilesetID::Bg);

    let mut grid = vec![];
    let start_x = (SPECS.render_width - (10 * SPECS.tile_width as u16)) as f32 / 2.0;
    for row in 0 .. 23 {
        for col in 0 .. 10 {
            let tile = world.add_entity(0);
            world.set_shape(tile, Shape::sprite_from_anim(TilesetID::Bg, 0));
            world.set_position(tile, Vec2{
                x: col as f32 * SPECS.tile_width as f32 + start_x,
                y: row as f32 * SPECS.tile_height as f32
            });
            grid.push(tile);
        }
    }

    let mut app = App::new(&world);
    
    // Main loop
    loop {
        app.start_frame(&mut world);
                
        // Update game and render entities 
        if mquad::is_key_down(mquad::KeyCode::LeftSuper) && mquad::is_key_pressed(mquad::KeyCode::Q) { break; }
        if mquad::is_key_pressed(mquad::KeyCode::A) { world.debug_atlas = !world.debug_atlas }
        if mquad::is_key_pressed(mquad::KeyCode::W) { world.debug_pivot = !world.debug_pivot }

        // Update
        for tile in grid.iter() {
            let mut pos = world.get_position(*tile);
            if pos.y > SPECS.render_height as f32 - SPECS.tile_height as f32 {
                pos.y = 0.0;
            }
            let inc = Vec2{
                x:0.0,
                y:0.05 * (((pos.x - start_x) / SPECS.tile_width as f32) + 1.0)
            };
            world.set_position(*tile, pos + inc);
            
        }
        world.framebuf.clear(Color24::green_dark());
        world.render_frame();

        // Overlay
        app.push_overlay(format!("FPS: {:.1}", 1.0 / world.time_elapsed()));
        app.push_overlay(format!("Entity count: {}", world.entities().len()));
        app.push_overlay(format!("Update time: {:.2}", world.time_update() * 1000.0));

        // Finish frame
        app.finish_frame(&mut world);
        mquad::next_frame().await;
    }
}


