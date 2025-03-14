use matte::{Frame, Side::*};
use macroquad::prelude::*;

#[macroquad::main("Frame Layout")]
async fn main() {
    loop {
        // Init frame
        clear_background(GRAY);
        let (width, height) = (screen_width(), screen_height());

        // Drawing helper function. Converts Rect types between
        // matte's and macroquad's, then draws it.
        fn draw_rect(rect: &matte::Rect<f32>, thickness: f32) {
            let rect = macroquad::math::Rect::new(rect.x, rect.y, rect.w, rect.h);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, thickness, BLACK);
        }

        // Init Layout. Prevents negative values.
        // You can optionally clamp it to  a minimum UI size.
        let mut root = Frame::new(matte::Rect {
            x: 10.0,
            y: 10.0,
            w: (width - 20.0).clamp(0.0, 8192.0),
            h: (height - 20.0).clamp(0.0, 8192.0),
        });

        // ----------------- Process Layout & Draw -----------------
        root.set_margin(8.0);
        root.set_scale(1.0);

        // Root rect
        draw_rect(&root.rect(), 4.0);

        // Left pane
        root.push(Left, 200.0, |pane| {
            draw_rect(&pane.rect(), 3.0);
            pane.set_gap(8.0);
            // Buttons
            for _n in 0..25 {
                pane.push(Top, 20.0, |button| {
                    draw_rect(&button.rect(), 2.0);
                });
            }
        });

        // Right Pane
        root.push(Right, 200.0, |pane| {
            draw_rect(&pane.rect(), 2.0);
        });

        // Middle Left
        root.fill(Left, 0.25, |pane| {
            draw_rect(&pane.rect(), 2.0);
        });

        // Middle Top
        root.fill(Top, 0.5, |pane| {
            draw_rect(&pane.rect(), 4.0);
            // Centered. Notice how 'side' here affects which side the available space
            // will be, in this caseit will be on top (the frame was added "from the bottom")
            pane.center(100.0, 100.0, |centered|{
                draw_rect(&centered.rect(), 2.0);
            });
            // Allows specifying extra offsets, width and height.
            pane.place(Left, 10.0, 10.0, 100.0, 20.0, |inner|{
                draw_rect(&inner.rect(), 2.0);
            });
        });

        // Middle Bottom
        root.fill(Bottom, 1.0, |pane| {
            draw_rect(&pane.rect(), 2.0);
            // Sized rect, will scale down preserving aspect
            pane.push_size(Bottom, 100.0, 50.0, |sized|{
                draw_rect(&sized.rect(), 2.0);
            });
        });

        // Present frame
        next_frame().await
    }
}
