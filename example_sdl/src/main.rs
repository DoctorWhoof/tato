use layframe::Frame;

fn main() -> Result<(), String> {
    let mut app = mini_sdl::App::new(
        "layout test",
        400,
        400,
        mini_sdl::Timing::Vsync,
        mini_sdl::Scaling::Fill,
        44100,
    )?;

    let mut font = app.font_load("example_sdl/src/classic-display.ttf", 16)?;

    while !app.quit_requested {
        // Init frame
        app.frame_start()?;
        app.canvas.set_draw_color((32, 32, 32, 255));
        app.canvas.clear();
        app.canvas.set_draw_color((0, 0, 0, 255));
        let (width, height) = (app.window_width() as u16, app.window_height() as u16);

        // Drawing helper function
        let mut draw_rect = |rect: &layframe::Rect, color: (u8, u8, u8), text: &'static str| {
            let rect = mini_sdl::sdl2::rect::Rect::new(
                rect.x as i32,
                rect.y as i32,
                rect.w as u32,
                rect.h as u32,
            );
            app.canvas.set_draw_color(color);
            app.canvas.fill_rect(rect).ok();
            app.canvas.set_draw_color((0, 0, 0, 255));
            app.canvas.draw_rect(rect).ok();
            // TODO: font shouldn't be mutable... move this function to App instead
            font.draw(text, rect.x + 4, rect.y + 4, 2.0, &mut app.canvas)
                .ok();
        };

        // Init Layout. "Saturating sub" clamps the result to 0, preventing overflow
        // (and a panic) if dimensions are too small.
        let w = (width * 2).saturating_sub(50);
        let h = (height * 2).saturating_sub(50);
        let mut root = Frame::root(25, 25, w, h);

        // Process Layout;
        draw_rect(&root.rect, (64, 64, 64), "");

        // Left panel
        let scale = 1.0;
        root.add_left(500, scale, |pane| {
            draw_rect(&pane.rect, (76, 88, 64), "left pane");
            // Buttons
            for _n in 0..25 {
                pane.add_top(40, scale, |child| {
                    draw_rect(&child.rect, (88, 96, 76), "button");
                });
            }
        });

        // Right Panel
        root.add_right(500, scale, |pane| {
            draw_rect(&pane.rect, (88, 76, 64), "right pane");
        });

        // Middle Panel
        root.fill_left(0.5, scale, |pane| {
            draw_rect(&pane.rect, (120, 130, 60), "middle pane left");
        });

        // Middle Panel
        root.fill_top(0.5, scale, |pane| {
            draw_rect(&pane.rect, (120, 130, 60), "middle pane top");
        });

        // Middle Panel
        root.fill_top(1.0, scale, |pane| {
            draw_rect(&pane.rect, (120, 130, 60), "middle pane bottom");
        });

        // Present frame
        app.frame_finish()?;
    }
    Ok(())
}
