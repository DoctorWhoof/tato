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
    
    if let Err(msg) = world.renderer.load_tileset(&atlas, TilesetID::Bg) {
        println!("{}", msg);
    }

    let mut grid = vec![];
    let columns = 30;
    let rows = 24;
    let start_x = (SPECS.render_width as usize - (columns * SPECS.tile_width as usize)) as f32 / 2.0;
    for row in 0 .. rows {
        for col in 0 .. columns {
            let tile = world.entity_add(0);
            world.set_shape(tile, Shape::sprite_from_anim(TilesetID::Bg, 2));
            world.set_position(tile, Vec2{
                x: col as f32 * SPECS.tile_width as f32 + start_x,
                y: row as f32 * SPECS.tile_height as f32
            });
            grid.push(tile);

            if let Some(ent) = world.get_entity_mut(tile){
                if let Shape::Sprite { ref mut flip_h, ref mut flip_v, .. } = ent.shape {
                    if row % 2 == 1 { *flip_v = true }
                    if col % 2 == 1 { *flip_h = true }
                }
            };
        }
    }

    let mut app = App::new(&world);
    
    // Main loop
    loop {
        app.start_frame(&mut world);
                
        // Update game and render entities 
        if mquad::is_key_down(mquad::KeyCode::LeftSuper) && mquad::is_key_pressed(mquad::KeyCode::Q) { break; }
        if mquad::is_key_pressed(mquad::KeyCode::A) { world.debug_atlas = !world.debug_atlas }
        if mquad::is_key_pressed(mquad::KeyCode::W) { world.debug_wireframe = !world.debug_wireframe }

        // Update
        for (i, tile) in grid.iter().enumerate() {
            let col = (i % columns) as f32;
            let row = (i / columns) as f32;
            world.set_position(*tile, Vec2 {
                x: (col * 8.0) + start_x,
                y: ((row + (world.time() * (col + 1.0) * 0.125)) * 8.0) % SPECS.render_height as f32
            })
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


