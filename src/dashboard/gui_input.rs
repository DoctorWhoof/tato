use super::*;

impl Dashboard {
    pub fn process_input(&mut self, backend: &impl Backend) {
        // Receive input
        if let Some(key) = backend.get_pressed_key() {
            match key {
                Key::None => {},
                Key::Text(ch) => {
                    if self.console_command_line.len() < COMMAND_MAX_LEN {
                        if ch >= 32 && ch < 128 {
                            self.console_command_line.push(&mut self.fixed_arena, ch).unwrap();
                        }
                    }
                },
                Key::Tab => {
                    self.display_debug_info = !self.display_debug_info;
                },
                Key::Grave => {
                    self.display_console = !self.display_console;
                },
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
                    if self.console_command_line.len() > 0 {
                        let text =
                            Text::from_buffer(&mut self.fixed_arena, &self.console_command_line)
                                .unwrap();
                        self.console_latest_command =
                            Command::parse_text(text.clone(), &self.fixed_arena);
                        self.console_buffer.push(&mut self.fixed_arena, text).unwrap();
                        self.console_command_line.clear();
                    }
                },
                Key::Backspace => {
                    if !self.console_command_line.is_empty() {
                        self.console_command_line.pop(&self.fixed_arena);
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
