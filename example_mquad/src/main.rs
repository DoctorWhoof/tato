use layframe::{Frame, Num, Side::*};
use macroquad::prelude::*;

#[macroquad::main("Frame Layout")]
async fn main() {
    let mut scale: f32 = 1.0;

    loop {
        // Init frame
        clear_background(BLACK);
        let (width, height) = (screen_width(), screen_height());

        // Input
        if is_key_pressed(KeyCode::Equal) {
            scale += 0.1;
        } else if is_key_pressed(KeyCode::Minus) {
            scale -= 0.1;
        } else if is_key_pressed(KeyCode::Key0) {
            scale = 1.0;
        }
        scale = scale.clamp(0.2, 2.0);

        // Drawing helper function
        let draw_rect = |rect: &layframe::Rect<f32>, color: [u8; 4], text: String| {
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
            draw_text_ex(text.as_str(), rect.x + 4.0, rect.y + 4.0, text_params);
        };

        // Init Layout. Prevents negative values.
        // You can optionally clamp it to  a minimum UI size.
        let mut root = Frame::new(layframe::Rect {
            x: 10.0,
            y: 10.0,
            w: (width - 20.0).clamp(0.0, 8192.0),
            h: (height - 20.0).clamp(0.0, 8192.0),
        });
        root.set_margin(4.0);
        root.set_scale(scale);

        // Process Layout;
        draw_rect(&root.rect(), [64, 64, 64, 255], "".to_string());
        // Left pane
        root.add(Left, 200.0, |pane| {
            draw_rect(&pane.rect(), [76, 88, 64, 255], "left pane".to_string());
            // Buttons
            for _n in 0..25 {
                pane.add(Top, 20.0, |button| {
                    draw_rect(&button.rect(), [88, 96, 76, 255], "button".to_string());
                    for _ in 0..25 {
                        button.add(Left, 10.0, |innie| {
                            draw_rect(&innie.rect(), [110, 130, 90, 255], "".to_string());
                        });
                    }
                });
            }
        });

        // Right Pane
        root.add(Right, 200.0, |pane| {
            draw_rect(&pane.rect(), [88, 76, 64, 255], "right pane".to_string());

            let count = 20;
            let gap_sum = pane.margin() * (count + 1) as f32;
            // Available space / count, but I subtract 1.0 to make it more stable
            // when resizing (avoids occasionally skipping last element)
            let button_size = (pane.rect().h - gap_sum - 1.0) / count as f32;

            for n in 0..count {
                pane.add(Top, button_size, |button| {
                    let text = format!("resizable button {}", n + 1);
                    draw_rect(&button.rect(), [120, 100, 90, 255], text);
                });
            }
        });

        // Middle Left
        root.fill(Left, 0.5, |pane| {
            draw_rect(&pane.rect(), [120, 130, 60, 255], "middle left".to_string());
        });

        // Middle Top
        root.fill(Top, 0.5, |pane| {
            draw_rect(&pane.rect(), [120, 130, 60, 255], "middle top".to_string());
            let ratio = 0.3;
            // Spiral rects!
            for _ in 0..3 {
                pane.fill(Top, ratio, |pane| {
                    draw_rect(&pane.rect(), [140, 160, 80, 255], "t".to_string());
                });
                pane.fill(Right, ratio, |pane| {
                    draw_rect(&pane.rect(), [140, 160, 80, 255], "r".to_string());
                });
                pane.fill(Bottom, ratio, |pane| {
                    draw_rect(&pane.rect(), [140, 160, 80, 255], "b".to_string());
                });
                pane.fill(Left, ratio, |pane| {
                    draw_rect(&pane.rect(), [140, 160, 80, 255], "l".to_string());
                });
            }
            pane.fill(Left, 1.0, |pane| {
                draw_rect(&pane.rect(), [180, 160, 80, 255], "end".to_string());
            });
        });

        // Middle Bottom
        root.fill(Bottom, 1.0, |pane| {
            add_fancy_panel(pane, |area| {
                area.add(Bottom, 20.0, |button| {
                    draw_rect(
                        &button.rect(),
                        [150, 170, 200, 255],
                        "bottom bar".to_string(),
                    );
                });
                for _ in 0..25 {
                    area.add(Top, 40.0, |button| {
                        draw_rect(&button.rect(), [150, 170, 90, 255], "test".to_string());
                    });
                }
            });
        });

        // Present frame
        next_frame().await
    }
}

fn add_fancy_panel<T>(frame: &mut Frame<T>, mut func: impl FnMut(&mut Frame<T>))
where
    T: Num,
{
    let text_size = 16.0;
    let text_params = TextParams {
        font_size: (text_size * frame.scale()) as u16,
        ..Default::default()
    };
    let rect = Rect::new(
        frame.rect().x.to_f32(),
        frame.rect().y.to_f32(),
        frame.rect().w.to_f32(),
        frame.rect().h.to_f32(),
    );
    let text_offset = Vec2::new(4.0, 12.0) * frame.scale();
    let bar = 16.0 * frame.scale();
    let text_width = text_size * 0.5 * "Fancy Custom Panel".chars().count() as f32 * frame.scale();
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
    frame.add(Top, T::from_f32(bar), |_| {});
    frame.fill(Top, 1.0, |content| func(content));
}
