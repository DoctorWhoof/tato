use macroquad::prelude::*;
use tato::*;
use tato_mquad::App;

fn window_conf() -> Conf {
    Conf {
        window_title: "Collision Test".into(),
        fullscreen: false,
        high_dpi: true,
        sample_count: 0,
        window_resizable: true,
        window_width: 320 * 3,
        window_height: 240 * 3,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // World specs
    let specs = Specs {
        render_width: 320,
        render_height: 240,
        atlas_width: 128,
        atlas_height: 128,
        tile_width: 8,
        tile_height: 8,
        colors_per_palette: 16,
    };

    // Can't init World without tileset and palette ids, so let's make some basic ones
    tileset_enum!{ TilesetID { Default } }
    palette_enum!{ PaletteID { Default } }

    // Spud init
    let mut world = World::<TilesetID, PaletteID>::new(specs);
    world.debug_colliders = true;   
    world.debug_pivot = true;   

    let ent_main = world.add_entity(0);
    let initial_position = tato::Vec2{x:160.0, y:100.0};
    world.set_position(ent_main, initial_position);
    let collider_point = Collider::from(tato::Vec2{x:0.0, y:0.0});
    let collider_rect = Collider::from(tato::Rect{x:-8.0, y:-8.0, w:16.0, h:16.0});
    world.add_collider(ent_main, collider_rect);

    let ent_rect_1 = world.add_entity(0);
    world.set_position(ent_rect_1, tato::Vec2::new(160.0, 120.0));
    world.add_collider(ent_rect_1, Collider::from(tato::Rect{x:0.0, y:0.0, w:32.0, h:32.0}));

    let ent_sine_x = world.add_entity(0);
    world.set_position(ent_sine_x, tato::Vec2::new(100.0, 60.0));
    world.add_collider(ent_sine_x, Collider::from(tato::Rect{x:0.0, y:0.0, w:32.0, h:16.0}));

    let ent_sine_y = world.add_entity(0);
    world.set_position(ent_sine_y, tato::Vec2::new(40.0, 120.0));
    world.add_collider(ent_sine_y, Collider::from(tato::Rect{x:0.0, y:0.0, w:32.0, h:16.0}));

    let ent_sine = world.add_entity(0);
    world.set_position(ent_sine, tato::Vec2::new(80.0, 120.0));
    world.add_collider(ent_sine, Collider::from(tato::Rect{x:0.0, y:0.0, w:32.0, h:16.0}));

    let ent_wall_top = world.add_entity(0);
    world.set_position(ent_wall_top, tato::Vec2::new(0.0, 0.0));
    world.add_collider(ent_wall_top, Collider::from(tato::Rect{x:0.0, y:0.0, w:320.0, h:16.0}));

    let ent_wall_bottom = world.add_entity(0);
    world.set_position(ent_wall_bottom, tato::Vec2::new(0.0, 224.0));
    world.add_collider(ent_wall_bottom, Collider::from(tato::Rect{x:0.0, y:0.0, w:320.0, h:16.0}));

    let ent_wall_left = world.add_entity(0);
    world.set_position(ent_wall_left, tato::Vec2::new(0.0, 16.0));
    world.add_collider(ent_wall_left, Collider::from(tato::Rect{x:0.0, y:0.0, w:16.0, h:208.0}));

    let ent_wall_right = world.add_entity(0);
    world.set_position(ent_wall_right, tato::Vec2::new(304.0, 16.0));
    world.add_collider(ent_wall_right, Collider::from(tato::Rect{x:0.0, y:0.0, w:16.0, h:208.0}));

    let speed = 120.0;
    let mut vel = tato::Vec2::zero();

    // main loop
    let mut app = App::new(&world);
    loop {
        app.start_frame(&mut world);

        // Update
        if is_key_down(KeyCode::LeftSuper) && is_key_pressed(KeyCode::Q) {
            break;
        }

        if is_key_pressed(KeyCode::Escape){
            world.set_position(ent_main, initial_position)
        }
                
        // Update
        if is_key_pressed(KeyCode::Key1){
            world.add_collider(ent_main, collider_point);
        } else if is_key_pressed(KeyCode::Key2){
            world.add_collider(ent_main, collider_rect);
        }

        if is_key_down(KeyCode::Up) {
            vel.y = -speed
        }else if is_key_down(KeyCode::Down) {
            vel.y = speed
        } else {
            vel.y = 0.0
        }

        if is_key_down(KeyCode::Left) {
            vel.x = -speed
        } else if is_key_down(KeyCode::Right) {
            vel.x = speed
        } else {
            vel.x = 0.0
        }

        // Moving colliders
        let oscillator = world.time() * 2.0;
        
        let sine_vel_x = tato::Vec2{x: oscillator.sin() * 60.0, y:0.0};
        world.move_with_collision(ent_sine_x, sine_vel_x, CollisionReaction::None);

        let sine_vel_y = tato::Vec2{x: 0.0, y:oscillator.sin() * 60.0};
        world.move_with_collision(ent_sine_y, sine_vel_y, CollisionReaction::None);

        let sine_vel = tato::Vec2{x: oscillator.sin() * 30.0, y:oscillator.cos() * 60.0};
        world.move_with_collision(ent_sine, sine_vel, CollisionReaction::None);

        // Static colliders
        world.use_static_collider(ent_rect_1);
        world.use_static_collider(ent_wall_top);
        world.use_static_collider(ent_wall_bottom);
        world.use_static_collider(ent_wall_left);
        world.use_static_collider(ent_wall_right);

        // Main Probe
        let collision = world.move_with_collision(ent_main, vel, CollisionReaction::Slide);  //TODO: not &mut, simply set vel to col.vel?
        if let Some(col) = &collision {
            vel = col.velocity
        }

        world.framebuf.clear(tato::Color24::gray_dark());
        world.render_frame();
        if let Some(col) = &collision {
            let pos = world.get_position(ent_main);
            let line_len = 10.0;
            let x1 = pos.x + (col.normal.x * line_len);
            let y1 = pos.y + (col.normal.y * line_len);
            world.framebuf.draw_line(pos.x as i32, pos.y as i32, x1 as i32, y1 as i32, tato::Color24::yellow(), 255);
            world.framebuf.draw_filled_rect(tato::Rect { x: pos.x as i32-1, y:pos.y as i32-1, w:3, h:3 }, tato::Color24::red());
        }

        // Debug Overlay
        app.push_overlay(format!("Update: {:.2?}", world.time_update() * 1000.0));
        app.push_overlay(format!("Vel: {:.2?}", vel));
        if let Some(col) = &collision {
            app.push_overlay(format!("Collision: {:.2?}", col));
        }

        // Finish
        app.finish_frame(&mut world);
        next_frame().await;
    }
}

