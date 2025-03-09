use layframe::Frame;
use macroquad::prelude::*;

#[macroquad::main("Frame Layout")]
async fn main() {
    let mut scale: f32 = 1.0;

    loop {
        // Init frame
        clear_background(BLACK);
        let (width, height) = (screen_width() as u16, screen_height() as u16);

        // Input
        if is_key_pressed(KeyCode::Equal) {
            scale += 0.2;
        } else if is_key_pressed(KeyCode::Minus) {
            scale -= 0.2;
        } else if is_key_pressed(KeyCode::Key0) {
            scale = 1.0;
        }
        scale = scale.clamp(0.2, 2.0);

        // Drawing helper function
        let draw_rect = |rect: &layframe::Rect, color: [u8; 4], text: &'static str| {
            let text_params = TextParams {
                font_size: (16.0 * scale) as u16,
                ..Default::default()
            };
            let rect = macroquad::math::Rect::new(
                rect.x as f32,
                rect.y as f32,
                rect.w as f32,
                rect.h as f32,
            );
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, color.into());
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, [0, 0, 0, 128].into());
            draw_text_ex(text, rect.x + 4.0, rect.y + 4.0, text_params);
        };

        // Init Layout. "Saturating sub" clamps the result to 0, preventing overflow
        // (and a panic) if dimensions are too small.
        let margin = 4;
        let mut root = Frame::new(
            layframe::Rect {
                x: 10,
                y: 10,
                w: width.saturating_sub(20),
                h: height.saturating_sub(20),
            },
            margin,
        );
        root.scale = scale;

        // Process Layout;
        draw_rect(&root.rect, [64, 64, 64, 255], "");

        // Left pane
        root.add_left(200, |pane| {
            draw_rect(&pane.rect, [76, 88, 64, 255], "left pane");
            // Buttons
            for _n in 0..25 {
                pane.add_top(20, |button| {
                    draw_rect(&button.rect, [88, 96, 76, 255], "button");
                    for _ in 0..25 {
                        button.add_left(10, |innie| {
                            draw_rect(&innie.rect, [110, 130, 90, 255], "");
                        });
                    }
                });
            }
        });

        // Right Pane
        root.add_right(200, |pane| {
            draw_rect(&pane.rect, [88, 76, 64, 255], "right pane");
            pane.add_bottom(20, |button| {
                draw_rect(&button.rect, [150, 170, 200, 255], "bottom");
            });
        });

        // Middle Left
        root.fill_left(0.5, |pane| {
            draw_rect(&pane.rect, [120, 130, 60, 255], "middle left");
        });

        // Middle Top
        root.fill_top(0.5, |pane| {
            draw_rect(&pane.rect, [120, 130, 60, 255], "middle top");
        });

        // Middle Bottom
        root.fill_bottom(1.0, |pane| {
            // draw_rect(&pane.rect, [120, 130, 60, 255], "middle bottom");

            add_fancy_panel(pane, |area| {
                area.add_bottom(20, |button| {
                    draw_rect(&button.rect, [150, 170, 200, 255], "bottom");
                });
                for _ in 0..25 {
                    area.add_top(40, |button| {
                        draw_rect(&button.rect, [150, 170, 90, 255], "test");
                    });
                }
            });
        });

        // Present frame
        next_frame().await
    }
}

fn add_fancy_panel(frame: &mut Frame, mut func: impl FnMut(&mut Frame)) {
    let text_size = 16.0;
    let text_params = TextParams {
        font_size: (text_size * frame.scale) as u16,
        ..Default::default()
    };
    let rect = Rect::new(
        frame.rect.x as f32,
        frame.rect.y as f32,
        frame.rect.w as f32,
        frame.rect.h as f32,
    );
    let text_offset = Vec2::new(4.0, 12.0) * frame.scale;
    let bar = 16.0 * frame.scale;
    let text_width = text_size * 0.5 * "Fancy Custom Panel".chars().count() as f32;
    if text_width < rect.w {
        draw_rectangle(
            rect.x,
            rect.y + bar,
            rect.w,
            rect.h - bar,
            [20, 20, 20, 255].into(),
        );
        draw_rectangle(
            rect.x,
            rect.y,
            text_width + text_offset.x,
            rect.h,
            [20, 20, 20, 255].into(),
        );
        draw_text_ex(
            "Fancy Custom Panel",
            rect.x + text_offset.x,
            rect.y + text_offset.y,
            text_params,
        );
    }
    frame.add_top(bar as u16, |_| {});
    frame.fill_top(1.0, |content| func(content));
}
