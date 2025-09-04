use super::*;

impl Dashboard {
    pub fn process_input<const LEN: usize>(
        &mut self,
        frame_arena: &mut Arena<LEN>,
        backend: &impl Backend,
    ) {
        let text_input = self.display_console && self.display_debug_info;

        // Receive input
        if let Some(key) = backend.get_pressed_key() {
            if self.display_console {
            } else {
            }
            match key {
                Key::None => {},
                Key::Text(ch) => {
                    if text_input
                        && self.console_command_line.len() < COMMAND_MAX_LEN as usize
                        && ch >= 32
                        && ch < 128
                    {
                        self.console_command_line.push(&mut self.fixed_arena, ch).unwrap();
                    }
                },
                Key::Tab => {
                    self.display_debug_info = !self.display_debug_info;
                },
                Key::Grave => {
                    self.display_console = !self.display_console;
                },
                Key::Plus => {
                    if text_input {
                        // Treat '+' as regular text input
                        if self.console_command_line.len() < COMMAND_MAX_LEN as usize {
                            self.console_command_line.push(&mut self.fixed_arena, b'+').unwrap();
                        }
                    } else {
                        if self.gui_scale < 10.0 {
                            self.gui_scale += 1.0
                        }
                    }
                },
                Key::Minus => {
                    if text_input {
                        // Treat '-' as regular text input
                        if self.console_command_line.len() < COMMAND_MAX_LEN as usize {
                            self.console_command_line.push(&mut self.fixed_arena, b'-').unwrap();
                        }
                    } else {
                        if self.gui_scale > 0.5 {
                            self.gui_scale -= 1.0
                        }
                    }
                },
                Key::Enter => {
                    if text_input && self.console_command_line.len() > 0 {
                        let text =
                            Text::from_buffer(&mut self.fixed_arena, &self.console_command_line)
                                .unwrap();
                        self.console_latest_command = Command::parse_str(
                            text.as_str(&self.fixed_arena).unwrap(), //  from fixed arena
                            frame_arena,                             // into shared arena!
                        );
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
