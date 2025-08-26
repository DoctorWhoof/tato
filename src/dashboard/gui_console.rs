use super::*;

impl<const LEN: usize> Dashboard<LEN> {
    pub fn process_console(&mut self, layout: &mut Frame<i16>, args: &DashArgs) {
        let font_size = self.font_size * self.gui_scale;

        // Receive input
        if args.display_console {
            match args.key {
                Key::None => {},
                Key::Text(ch) => {
                    if (self.console_line_len as usize) < COMMAND_MAX_LEN {
                        if ch >= 32 && ch < 128 {
                            self.console_line_buffer[self.console_line_len as usize] = ch;
                            self.console_line_len += 1;
                        }
                    }
                },
                Key::Tab => {},
                Key::Plus => {
                    if self.gui_scale < 10.0 {
                        self.gui_scale += 1.0
                    }
                },
                Key::Minus => {
                    if self.gui_scale > 0.5 {
                        self.gui_scale -= 1.0
                    }
                },
                Key::Enter => {
                    if self.console_line_len > 0 {
                        // Strip extra characters
                        let line: [u8; COMMAND_MAX_LEN] = from_fn(|i| {
                            if i < self.console_line_len as usize {
                                self.console_line_buffer[i]
                            } else {
                                0
                            }
                        });
                        self.console_buffer.push(&mut self.fixed_arena, line).unwrap();
                        self.console_line_len = 0;
                    }
                },
                Key::Backspace => {
                    if self.console_line_len > 0 {
                        self.console_line_len -= 1;
                    }
                },
                Key::Delete => todo!(),
                Key::Left => todo!(),
                Key::Right => todo!(),
                Key::Up => todo!(),
                Key::Down => todo!(),
            }
        }

        // Console
        if args.display_console {
            layout.push_edge(Edge::Bottom, 80, |console| {
                console.set_margin(5);

                // draw BG rect
                let op_handle = self
                    .temp_arena
                    .alloc(DrawOp::Rect {
                        rect: console.rect(),
                        color: RGBA32 { r: 18, g: 18, b: 18, a: 230 },
                    })
                    .unwrap();
                self.ops.push(&mut self.temp_arena, op_handle).unwrap();

                // Draw main console line text
                let text_result = Text::from_ascii(&mut self.temp_arena, &self.console_line_buffer);
                if let Ok(text) = text_result {
                    console.push_edge(Edge::Bottom, self.font_size as i16, |line| {
                        let rect = line.rect();
                        let text_op_id = self
                            .temp_arena
                            .alloc(DrawOp::Text {
                                text: text.clone(),
                                x: rect.x,
                                y: rect.y,
                                size: font_size,
                                color: RGBA32::WHITE,
                            })
                            .unwrap();
                        self.ops.push(&mut self.temp_arena, text_op_id).unwrap();
                    });
                }

                // Draw console buffer (previous lines)
                let remaining_rect = console.rect();
                for text in self.console_buffer.items(&self.fixed_arena).unwrap().rev() {
                    let mut line_rect = Rect::<i16>::default();
                    console.push_edge(Edge::Bottom, self.font_size as i16, |line| {
                        line_rect = line.rect();
                        let text = Text::from_ascii(&mut self.temp_arena, text).unwrap();
                        let op_id = self
                            .temp_arena
                            .alloc(DrawOp::Text {
                                text,
                                x: line_rect.x,
                                y: line_rect.y,
                                size: font_size,
                                color: RGBA32::WHITE,
                            })
                            .unwrap();
                        self.ops.push(&mut self.temp_arena, op_id).unwrap();
                    });
                    if line_rect.y < remaining_rect.y + self.font_size as i16 {
                        break;
                    }
                }
            });
        }
    }
}
