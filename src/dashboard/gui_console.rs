use super::*;

impl Dashboard {
    pub fn process_console<const LEN: usize>(
        &mut self,
        layout: &mut Frame<i16>,
        frame_arena: &mut Arena<LEN>,
    ) {
        let font_size = self.font_size * self.gui_scale;

        // Console
        if self.display_console {
            layout.push_edge(Edge::Bottom, (font_size * 2.0) as i16, |console| {
                console.set_margin(5);

                // draw BG rect
                let op_handle = frame_arena
                    .alloc(DrawOp::Rect {
                        rect: console.rect(),
                        color: RGBA32 { r: 18, g: 18, b: 18, a: 230 },
                    })
                    .unwrap();
                self.ops.push(frame_arena, op_handle).unwrap();

                // Draw main console line text
                let text_slice = self.console_command_line.as_slice(&self.fixed_arena);
                let slice = text_slice.unwrap_or(&[' ' as u8]);
                let text_result = Text::from_ascii(frame_arena, slice);
                let text = text_result.unwrap_or(Text::default());
                console.push_edge(Edge::Bottom, self.font_size as i16, |line| {
                    let rect = line.rect();
                    let text_op_id = frame_arena
                        .alloc(DrawOp::Text {
                            text: text.clone(),
                            x: rect.x,
                            y: rect.y,
                            size: font_size,
                            color: RGBA32::WHITE,
                        })
                        .unwrap();
                    self.ops.push(frame_arena, text_op_id).unwrap();
                });

                // Draw console buffer (previous lines)
                let remaining_rect = console.rect();
                for text in self.console_buffer.items(&self.fixed_arena).unwrap().rev() {
                    let mut line_rect = Rect::<i16>::default();
                    console.push_edge(Edge::Bottom, self.font_size as i16, |line| {
                        line_rect = line.rect();
                        // Copy from fixed arena to frame arena, since the
                        // latter is shared with the backend
                        let copied_text = Text::from_str(
                            frame_arena,
                            text.as_str(&self.fixed_arena).unwrap(), //
                        );
                        let op_id = frame_arena.alloc(DrawOp::Text {
                            text: copied_text.unwrap(),
                            x: line_rect.x,
                            y: line_rect.y,
                            size: font_size,
                            color: RGBA32::WHITE,
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
}
