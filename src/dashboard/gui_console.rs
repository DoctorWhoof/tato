use super::*;

impl Dashboard {
    pub(super) fn process_console<const LEN: usize>(
        &mut self,
        tato: &Tato,
        layout: &mut Frame<i16>,
        frame_arena: &mut Arena<LEN>,
    ) {
        // Console
        if !self.display_console {
            return;
        }
        layout.fitting = Fitting::Clamp;
        let font_size = self.font_size * self.gui_scale;
        let rect = layout.rect(); //.shrink(10);
        let w = (rect.w as f32 / self.gui_scale) as i16;

        layout.place(Align::Center, 0, 0, w, font_size as i16 * 6, |console| {
            console.set_margin(5);
            console.set_gap(0);

            // draw BG rect
            let op_handle = frame_arena
                .alloc(DrawOp::Rect {
                    rect: console.rect(),
                    color: RGBA32 { r: 18, g: 18, b: 18, a: 230 },
                })
                .unwrap();
            self.ops.push(frame_arena, op_handle).unwrap();

            // Draw main console line text
            let command_line_bytes = self.console_line_buffer.as_slice(&self.fixed_arena).unwrap();
            let prompt = if tato.time().fract() < 0.5 { [b' '] } else { [b'_'] };
            let text_result = Text::join_bytes(
                frame_arena, //
                &[b"command: ", command_line_bytes, &prompt],
            );
            let text = text_result.unwrap_or(Text::default());

            console.push_edge(Edge::Bottom, self.font_size as i16, |line| {
                let rect = line.rect();
                let text_op_id = frame_arena
                    .alloc(DrawOp::Text {
                        text: text.clone(),
                        x: rect.x + font_size as i16,
                        y: rect.y,
                        size: font_size,
                        color: RGBA32::WHITE,
                    })
                    .unwrap();
                self.ops.push(frame_arena, text_op_id).unwrap();
            });

            // Draw console buffer (previous lines)
            let remaining_rect = console.rect();
            for text in self.console_buffer.items(&self.fixed_arena).rev() {
                let mut line_rect = Rect::<i16>::default();
                console.push_edge(Edge::Bottom, self.font_size as i16, |line| {
                    line_rect = line.rect();
                    // Copy from fixed arena to frame arena, since the
                    // latter is shared with the backend
                    let copied_text = Text::from_bytes(frame_arena, text);
                    let op_id = frame_arena.alloc(DrawOp::Text {
                        text: copied_text.unwrap(),
                        x: line_rect.x + font_size as i16,
                        y: line_rect.y,
                        size: font_size,
                        color: RGBA32::GRAY,
                    });
                    self.ops.push(frame_arena, op_id.unwrap()).unwrap();
                });
                if line_rect.y < remaining_rect.y + self.font_size as i16 {
                    break;
                }
            }
        });
    }
}
