use super::*;

impl Dashboard {
    pub fn draw_tooltip<const LEN: usize>(&mut self, frame_arena: &mut Arena<LEN, u32>, backend: &impl Backend) {
        // Generate tooltip command
        if !self.mouse_over_text.is_empty() {
            let mouse = backend.get_mouse();
            let pad = self.gui_scale as i16;
            // TODO: Need a way to calculate font size... without knowing what the font is!
            // Maybe just a multiplier, or maybe even only work with monospaced fonts?
            let width = ((self.font_size / 1.9 * self.mouse_over_text.len() as f32)
                * self.gui_scale) as i16;
            let font_size = 12.0 * self.gui_scale as f32;

            let text_x = mouse.x - width - 12;
            let text_y = mouse.y + 12;

            // Background
            let black = RGBA32 { r: 0, g: 0, b: 0, a: 255 };
            let handle = frame_arena
                .alloc(DrawOp::Rect {
                    rect: Rect {
                        x: text_x - pad,
                        y: text_y,
                        w: width + pad + pad,
                        h: font_size as i16,
                    },
                    color: black,
                })
                .unwrap();
            self.ops.push(frame_arena, handle).expect("Dashboard: Can't insert mouse-over rect ");

            // Text
            let white = RGBA32 { r: 255, g: 255, b: 255, a: 255 };
            let handle = frame_arena
                .alloc(DrawOp::Text {
                    text: self.mouse_over_text.clone(),
                    x: text_x,
                    y: text_y,
                    size: font_size,
                    color: white,
                })
                .unwrap();
            self.ops.push(frame_arena, handle).expect("Dashboard: Can't insert mouse-over text ");
        }
    }
}
