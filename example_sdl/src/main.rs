use layframe::{Direction, Layout};
use mini_sdl::sdl2::rect::Rect;

// TODO: "Fill" mini_sdl draw mode (target is resized to window size whenever a window change event occurs)
// TODO: Proceed with Layout methods (double ended cursor, figure out rect sizes)

fn main() -> Result<(), String> {
    let mut app = mini_sdl::App::new(
        "layout test",
        400,
        400,
        mini_sdl::Timing::Vsync,
        mini_sdl::Scaling::Fill,
        44100,
    )?;

    use Direction::*;

    while !app.quit_requested {
        // Init frame
        app.frame_start()?;
        app.canvas.set_draw_color((64, 64, 64, 255));
        app.canvas.clear();
        app.canvas.set_draw_color((0, 0, 0, 255));

        // Skip if window is too small (prevents subtraction overflow)
        if app.window_width() > 50 && app.window_height() > 50 {
            // Process Layout
            let lay = Layout::<&'static str>::new()
                .width((app.window_width() as u16 * 2) - 50)
                .height((app.window_height() as u16 * 2) - 50)
                .horizontal()
                .push_left(50, "left panel A")
                .push_left(50, "left panel B");
            // .push_end(50, "right panel");
            // let lay = Layout {
            //     width: ,
            //     height: ,
            //     scale: 1.0,
            //     root: Frame {
            //         children: Some(vec![
            //             Frame::from_start(50, Horizontal),
            //             Frame::from_end(50, Horizontal),
            //         ]),
            //         ..Default::default()
            //     },
            // };
            // lay.update();

            // Draw
            let rect = Rect::new(25, 25, lay.width as u32, lay.height as u32);
            app.canvas.draw_rect(rect).ok();
        }
        app.frame_finish()?;
    }
    Ok(())
}
