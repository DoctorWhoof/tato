use layframe::Frame;
use macroquad::prelude::*;

#[macroquad::main("Frame Layout")]
async fn main() {
    let font = load_ttf_font("example_sdl/src/classic-display.ttf")
        .await
        .unwrap();

    let text_params = TextParams {
        font: Some(&font),
        font_size: 16,
        font_scale: 1.0,
        font_scale_aspect: 1.0,
        rotation: 0.0,
        color: Color::new(1.0, 1.0, 1.0, 1.0),
    };

    let mut scale:f32 = 1.0;

    loop {
        // Init frame
        clear_background(BLACK);
        let (width, height) = (screen_width() as u16, screen_height() as u16);

        if is_key_pressed(KeyCode::Equal){
            scale += 0.2;
        } else if is_key_pressed(KeyCode::Minus){
            scale -= 0.2;
        }

        scale = scale.clamp(0.2, 4.0);

        // Drawing helper function
        let draw_rect = |rect: &layframe::Rect, color: [u8; 4], text: &'static str| {
            let text_params = text_params.clone();
            let rect = macroquad::math::Rect::new(
                rect.x as f32,
                rect.y as f32,
                rect.w as f32,
                rect.h as f32,
            );
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, color.into());
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, [0, 0, 0, 255].into());
            draw_text_ex(text, rect.x + 4.0, rect.y + 4.0, text_params);
        };

        // Init Layout. "Saturating sub" clamps the result to 0, preventing overflow
        // (and a panic) if dimensions are too small.
        let w = width.saturating_sub(20);
        let h = height.saturating_sub(20);
        let mut root = Frame::root(10, 10, w, h);
        root.margin = Some(5);
        root.scale = scale;

        // Process Layout;
        draw_rect(&root.rect, [64, 64, 64, 255], "");

        // Left pane
        root.add_left(200, |pane| {
            draw_rect(&pane.rect, [76, 88, 64, 255], "left pane");
            // Buttons
            for _n in 0..25 {
                pane.add_top(40, |button| {
                    draw_rect(&button.rect, [88, 96, 76, 255], "button");
                });
            }
        });

        // Right Pane
        root.add_right(200, |pane| {
            draw_rect(&pane.rect, [88, 76, 64, 255], "right pane");
        });

        // Middle Pane
        root.fill_left(0.5, |pane| {
            draw_rect(&pane.rect, [120, 130, 60, 255], "middle pane left");
        });

        // Middle Pane
        root.fill_top(0.5, |pane| {
            draw_rect(&pane.rect, [120, 130, 60, 255], "middle pane top");
        });

        // Middle Panel
        root.fill_top(1.0, |pane| {
            draw_rect(&pane.rect, [120, 130, 60, 255], "middle pane bottom");
        });

        // Present frame
        next_frame().await
    }
}
