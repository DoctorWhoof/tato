use super::*;

impl Dashboard {
    pub fn process_input(&mut self, backend: &impl Backend) {
        // Receive input
        if let Some(key) = backend.get_pressed_key() {
            match key {
                Key::None => {},
                Key::Text(ch) => {
                    if self.console_line_buffer.len() < COMMAND_MAX_LEN {
                        if ch >= 32 && ch < 128 {
                            self.console_line_buffer.push(&mut self.fixed_arena, ch).unwrap();
                        }
                    }
                },
                Key::Tab => self.display_debug_info = !self.display_debug_info,
                Key::Grave => self.display_console = !self.display_console,
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
                    if self.console_line_buffer.len() > 0 {
                        // Strip extra characters
                        let text =
                            Text::from_buffer(&mut self.fixed_arena, &self.console_line_buffer)
                                .unwrap();
                        // let line: [u8; COMMAND_MAX_LEN] = from_fn(|i| {
                        //     if i < self.console_line_buffer.len() {
                        //         buffer[i as usize] //
                        //     } else {
                        //         0
                        //     }
                        // });
                        self.console_buffer.push(&mut self.fixed_arena, text).unwrap();
                        self.console_line_buffer.clear();
                    }
                },
                Key::Backspace => {
                    if !self.console_line_buffer.is_empty() {
                        self.console_line_buffer.pop(&self.fixed_arena);
                    }
                },
                Key::Delete => {},
                Key::Left => {},
                Key::Right => {},
                Key::Up => {},
                Key::Down => {},
            }
        }
    }
}
