// #![no_std]
use mini_sdl::*;
use videochip::*;
use shared_game::*;


fn main() -> SdlResult<()> {
    let mut vid = VideoChip::new(240, 180);

    let mut app = mini_sdl::App::new(
        "Basic Game",
        // 240, 180, // cropped to 240,180 option
        vid.width(),
        vid.height(),
        mini_sdl::Timing::Vsync,
        Scaling::PreserveAspect,
        None,
    )?;
    // let debug_font = app.font_load("src/debug/font.ttf", 32.0, 1.0)?;
    let debug_font = app.font_load("_shared_game/fonts/font.ttf", 32)?;
    app.print_fps_interval = Some(1.0);
    app.display_overlay = true;
    app.default_font = Some(debug_font);

    // app.init_pixel_buffer()?;

    // let mut scene = Scene::A(CameraScrolling::new(&mut vid));
    let mut scene = Scene::C(MinimalScene::new(&mut vid));

    while !app.quit_requested {
        // let time = Instant::now();
        app.frame_start()?;

        let state = AppState {
            pad: app.pad.buttons,
            time: app.time(),
            elapsed: app.elapsed_time(),
        };

        // Scene switching and update.
        let mode_request = match &mut scene {
            Scene::A(scn) => scn.update(&mut vid, state),
            Scene::B(scn) => scn.update(&mut vid, state),
            Scene::C(scn) => scn.update(&mut vid, state),
        };

        // Dump
        // for line in &vid.scanlines {
        //     println!("{:?}", line);
        // }
        // break;

        // Copy pixels to app
        let width = app.width() as usize;
        app.pixel_buffer_update(|pixel, _pitch| {
            for (color, coords) in vid.iter_pixels() {
                // Each pixel_buffer pixel is 3 bytes
                let i = ((coords.y as usize * width) + coords.x as usize) * 3;

                // // Cropped to 240 x 180 option
                // if coords.x < 8 || coords.x >= 248 || coords.y < 8 || coords.y >= 188 {
                //     continue;
                // }
                // let mapped_x = coords.x as usize - 8;
                // let mapped_y = coords.y as usize - 8;
                // let i = ((mapped_y * width) + mapped_x) * 3;

                pixel[i] = color.r;
                pixel[i + 1] = color.g;
                pixel[i + 2] = color.b;
            }
        })?;

        // Scenes request a new scene if their return is Some(mode).
        // Processed after copying pixels for this frame.
        if let Some(mode) = mode_request {
            vid.reset_all();
            match mode {
                Mode::A => scene = Scene::A(CameraScrolling::new(&mut vid)),
                Mode::B => scene = Scene::B(FixedCamera::new(&mut vid)),
                Mode::C => scene = Scene::C(MinimalScene::new(&mut vid)),
            }
        }

        // Present and finish
        app.pixel_buffer_present()?;
        // app.overlay_push(format!("update time: {:.1} ms", app.update_time() * 1000.0));
        // println!("{:.1} ms", time.elapsed().as_secs_f64() * 1000.0);
        // println!("{:.1} ms", app.update_time() * 1000.0);

        app.frame_finish()?;
        // println!("{:.3} ms", app.update_time() * 1000.0);
    }
    Ok(())
}
