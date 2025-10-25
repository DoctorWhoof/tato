use super::*;

/// Simple text wrapper around a TempID<[u8]>
#[derive(Debug, Clone, Copy)]
pub struct TempText<Idx: Copy = u32> {
    slice_id: TempID<[u8], Idx>,
}

impl<Idx: ArenaIndex + Copy> TempText<Idx> {
    /// Create text from a string slice
    pub fn from_str<const LEN: usize>(
        arena: &mut TempArena<LEN, Idx>,
        s: &str,
    ) -> TempArenaResult<Self> {
        let slice_id = arena.alloc_slice_from_iter(s.bytes())?;
        Ok(Self { slice_id })
    }

    /// Create text from bytes
    pub fn from_bytes<const LEN: usize>(
        arena: &mut TempArena<LEN, Idx>,
        bytes: &[u8],
    ) -> TempArenaResult<Self> {
        let slice_id = arena.alloc_slice_from_iter(bytes.iter().copied())?;
        Ok(Self { slice_id })
    }

    /// Get length in bytes
    pub fn len(&self) -> usize {
        self.slice_id.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get as byte slice
    pub fn as_bytes<'a, const LEN: usize>(&self, arena: &'a TempArena<LEN, Idx>) -> Option<&'a [u8]> {
        arena.get_slice(self.slice_id)
    }

    /// Get as string slice (validates UTF-8)
    pub fn as_str<'a, const LEN: usize>(&self, arena: &'a TempArena<LEN, Idx>) -> Option<&'a str> {
        let bytes = arena.get_slice(self.slice_id)?;
        core::str::from_utf8(bytes).ok()
    }

    /// Format function supporting multiple values and precision formatting
    /// Supports: "{}", "{:.N}", "{:?}" with multiple values
    pub fn format_display<const LEN: usize, V>(
        arena: &mut TempArena<LEN, Idx>,
        template: &str,
        values: &[V],
    ) -> TempArenaResult<Self>
    where
        V: core::fmt::Display,
    {
        // Collect all formatted parts into a temporary buffer
        let mut output_buffer = [0u8; 512];
        let mut output_pos = 0;
        let mut template_pos = 0;
        let mut value_index = 0;

        while template_pos < template.len() {
            if let Some(placeholder_start) = template[template_pos..].find('{') {
                let abs_start = template_pos + placeholder_start;
                
                // Copy text before placeholder
                let before_bytes = template[template_pos..abs_start].as_bytes();
                if output_pos + before_bytes.len() > output_buffer.len() {
                    return Err(TempArenaError::InvalidBounds);
                }
                output_buffer[output_pos..output_pos + before_bytes.len()].copy_from_slice(before_bytes);
                output_pos += before_bytes.len();

                // Find end of placeholder
                if let Some(placeholder_end) = template[abs_start..].find('}') {
                    let abs_end = abs_start + placeholder_end;
                    let placeholder = &template[abs_start..abs_end + 1];
                    
                    // Format the value based on placeholder type
                    if value_index < values.len() {
                        let formatted_len = if placeholder.contains(":.") {
                            // Precision formatting like {:.3}
                            if let Some(precision_str) = extract_precision(placeholder) {
                                format_with_precision(&values[value_index], precision_str, 
                                                   &mut output_buffer[output_pos..])
                            } else {
                                format_simple(&values[value_index], &mut output_buffer[output_pos..])
                            }
                        } else {
                            // Simple {} formatting
                            format_simple(&values[value_index], &mut output_buffer[output_pos..])
                        };
                        
                        output_pos += formatted_len;
                        value_index += 1;
                    }
                    
                    template_pos = abs_end + 1;
                } else {
                    break;
                }
            } else {
                // Copy remaining text
                let remaining_bytes = template[template_pos..].as_bytes();
                if output_pos + remaining_bytes.len() > output_buffer.len() {
                    return Err(TempArenaError::InvalidBounds);
                }
                output_buffer[output_pos..output_pos + remaining_bytes.len()].copy_from_slice(remaining_bytes);
                output_pos += remaining_bytes.len();
                break;
            }
        }

        // Create the final text from the formatted buffer
        let slice_id = arena.alloc_slice_from_fn(output_pos, |i| output_buffer[i])?;
        Ok(Self { slice_id })
    }

    /// Simple format function for single value - kept for compatibility
    pub fn format<const LEN: usize, V>(
        arena: &mut TempArena<LEN, Idx>,
        template: &str,
        value: V,
    ) -> TempArenaResult<Self>
    where
        V: core::fmt::Display,
    {
        Self::format_display(arena, template, &[value])
    }
}

// Extract precision from format specifier like {:.3}
fn extract_precision(placeholder: &str) -> Option<&str> {
    if let Some(dot_pos) = placeholder.find(":.") {
        let after_dot = &placeholder[dot_pos + 2..];
        if let Some(brace_pos) = after_dot.find('}') {
            return Some(&after_dot[..brace_pos]);
        }
    }
    None
}

// Format a value with precision into buffer, returns bytes written
fn format_with_precision<V: core::fmt::Display>(
    value: &V, 
    precision_str: &str, 
    buffer: &mut [u8]
) -> usize {
    use core::fmt::Write;
    
    struct BufferWriter<'a> {
        buffer: &'a mut [u8],
        pos: usize,
    }
    
    impl<'a> Write for BufferWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            if self.pos + bytes.len() > self.buffer.len() {
                return Err(core::fmt::Error);
            }
            
            self.buffer[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
            self.pos += bytes.len();
            Ok(())
        }
    }
    
    let mut writer = BufferWriter { buffer, pos: 0 };
    
    // Try to parse precision and format accordingly
    if let Ok(precision) = precision_str.parse::<usize>() {
        let _ = write!(writer, "{:.prec$}", value, prec = precision);
    } else {
        let _ = write!(writer, "{}", value);
    }
    
    writer.pos
}

// Simple formatting without precision, returns bytes written
fn format_simple<V: core::fmt::Display>(value: &V, buffer: &mut [u8]) -> usize {
    use core::fmt::Write;
    
    struct BufferWriter<'a> {
        buffer: &'a mut [u8],
        pos: usize,
    }
    
    impl<'a> Write for BufferWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            if self.pos + bytes.len() > self.buffer.len() {
                return Err(core::fmt::Error);
            }
            
            self.buffer[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
            self.pos += bytes.len();
            Ok(())
        }
    }
    
    let mut writer = BufferWriter { buffer, pos: 0 };
    let _ = write!(writer, "{}", value);
    writer.pos
}