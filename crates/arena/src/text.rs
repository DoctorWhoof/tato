//! Text utilities for Slice<u8> - treating byte slices as strings
mod debug_buffer;
use debug_buffer::*;

use crate::{Arena, ArenaErr, ArenaIndex, ArenaOps, ArenaRes, Buffer, Slice};
use core::fmt::Write;

/// Text stored as bytes in the arena
#[derive(Debug, Clone)]
pub struct Text<I = u32> {
    pub slice: Slice<u8, I>,
}

/// Unnalocated Text (will fail any attempt to obtain its data from an arena)
impl<I> Default for Text<I>
where
    I: ArenaIndex,
{
    fn default() -> Self {
        Self { slice: Default::default() }
    }
}

/// Implementation for "Slice<u8>"" specifically to add text functionality. You can convert to and from
/// &str, and use [Text::format] for very basic message+value formatting in no_std environments.
impl<I> Text<I>
where
    I: ArenaIndex,
{
    /// Get the length of the text in bytes
    pub fn len(&self) -> usize {
        self.slice.len().to_usize()
    }

    /// Check if the text is empty
    pub fn is_empty(&self) -> bool {
        self.slice.len() == I::zero()
    }

    /// Get the text directly as a u8 slice
    pub fn as_slice<'a, const LEN: usize>(
        &self,
        arena: &'a Arena<LEN, I>,
    ) -> ArenaRes<&'a [u8]> {
        arena.get_slice(self.slice.clone())
    }

    /// Get the text as &str (requires arena for safety)
    /// Returns an Error if the bytes are not valid UTF-8
    pub fn as_str<'a, const LEN: usize>(&self, arena: &'a Arena<LEN, I>) -> ArenaRes<&'a str> {
        let bytes = arena.get_slice(self.slice.clone())?;
        if let Ok(value) = core::str::from_utf8(bytes) {
            Ok(value)
        } else {
            Err(ArenaErr::InvalidUTF8)
        }
    }

    /// Create text from a string slice
    pub fn from_str<const LEN: usize>(arena: &mut Arena<LEN, I>, s: &str) -> ArenaRes<Self> {
        let bytes = s.as_bytes();
        let len = s.len();
        let slice = arena.alloc_slice_from_fn(len, |i| bytes[i])?;
        Ok(Self { slice })
    }

    /// Create text from a Buffer<u8>
    pub fn from_buffer<const LEN: usize>(
        arena: &mut Arena<LEN, I>,
        buffer: &Buffer<u8, I>,
    ) -> ArenaRes<Self> {
        let used_len = I::from_usize_checked(buffer.len()).ok_or(ArenaErr::IndexConversion)?;
        let used_len_usize = used_len.to_usize();
        
        // Copy to temporary buffer to avoid borrow issues
        let mut temp = [0u8; 4096];
        if used_len_usize > 4096 {
            return Err(ArenaErr::CapacityExceeded);
        }
        
        {
            let slice_content = arena.get_slice(buffer.slice.clone())?;
            temp[..used_len_usize].copy_from_slice(&slice_content[..used_len_usize]);
        }
        
        // Now allocate from the temporary buffer
        let slice = arena.alloc_slice(&temp[..used_len_usize])?;
        Ok(Self { slice })
    }

    /// Create text from a valid (but non-zero) ASCII slice
    pub fn from_bytes<const LEN: usize>(
        arena: &mut Arena<LEN, I>,
        bytes: &[u8],
    ) -> ArenaRes<Self> {
        let mut len = 0;
        for i in 0..bytes.len() {
            let value = bytes[i];
            if value > 0 {
                if value.is_ascii() {
                    len = i + 1;
                } else {
                    return Result::Err(ArenaErr::InvalidUTF8);
                }
            } else {
                break;
            }
        }
        let slice = arena.alloc_slice_from_fn(len, |i| bytes[i])?;
        Ok(Self { slice })
    }

    /// Create text from a function that generates bytes
    pub fn from_fn<const LEN: usize, F>(
        arena: &mut Arena<LEN, I>,
        length: usize,
        func: F,
    ) -> ArenaRes<Self>
    where
        F: Fn(usize) -> u8,
    {
        let slice = arena.alloc_slice_from_fn(length, func)?;
        Ok(Self { slice })
    }

    /// Join multiple text instances into a single text. Will fail for lengths over 1Kb.
    pub fn join<const LEN: usize>(
        arena: &mut Arena<LEN, I>,
        sources: &[Text<I>],
    ) -> ArenaRes<Self> {
        if sources.is_empty() {
            return Ok(Self::default());
        }

        // Calculate total length first
        let mut total_len = 0usize;
        for text in sources {
            total_len += text.slice.len().to_usize();
        }
        let final_len = total_len;

        // Use a temp arena to collect all the bytes, then copy to dest arena
        let mut temp_arena = Arena::<1024, I>::new();
        let temp_slice = temp_arena.alloc_slice_from_fn(final_len, |i| {
            // Find which source this byte belongs to
            let mut offset = 0usize;
            for text in sources {
                let text_len = text.slice.len().to_usize();
                if i < offset + text_len {
                    // Get the byte from this text
                    let bytes = arena.get_slice(text.slice.clone()).unwrap();
                    return bytes[i - offset];
                }
                offset += text_len;
            }
            0 // Should never reach here
        })?;

        // Now copy from temp arena to dest arena
        let temp_bytes = temp_arena.get_slice(temp_slice.clone())?;
        let slice = arena.alloc_slice_from_fn(final_len, |i| temp_bytes[i])?;
        Ok(Self { slice })
    }

    /// Join multiple byte slices into a single text.
    pub fn join_bytes<const LEN: usize>(
        arena: &mut Arena<LEN, I>,
        slices: &[&[u8]],
    ) -> ArenaRes<Self> {
        if slices.is_empty() {
            let empty_slice = arena.alloc_slice::<u8>(&[])?;
            return Ok(Self { slice: empty_slice });
        }

        // Calculate total length first
        let mut total_len = 0usize;
        for slice in slices {
            total_len += slice.len();
        }
        let final_len = total_len;

        // Allocate and fill the result slice
        let slice = arena.alloc_slice_from_fn(final_len, |i| {
            // Find which source slice this byte belongs to
            let mut offset = 0usize;
            for slice in slices {
                if i < offset + slice.len() {
                    return slice[i - offset];
                }
                offset += slice.len();
            }
            0 // Should never reach here
        })?;

        Ok(Self { slice })
    }

    /// Create formatted text using Debug trait
    /// Replaces "{:?}" placeholders with values by index, with message2 appended after
    pub fn format_dbg<const LEN: usize, M1, M2, V>(
        arena: &mut Arena<LEN, I>,
        message1: M1,
        values: &[V],
        message2: M2,
    ) -> ArenaRes<Self>
    where
        M1: AsRef<str>,
        M2: AsRef<str>,
        V: core::fmt::Debug,
    {
        let message1_str = message1.as_ref();
        let message2_str = message2.as_ref();
        debug_assert!(message1_str.is_ascii());
        debug_assert!(message2_str.is_ascii());

        // Validate format string first
        if let Err(err_msg) = Self::validate_format_string(message1_str) {
            panic!("Debug format error: {} in: '{}'", err_msg, message1_str);
        }

        // Check placeholder count matches value count
        let placeholder_count = Self::count_placeholders(message1_str);
        if placeholder_count != values.len() {
            panic!(
                "Debug format placeholder mismatch: found {} placeholders but {} values provided in: '{}'",
                placeholder_count,
                values.len(),
                message1_str
            );
        }

        // Format using debug trait with indexed values
        let mut debug_buf = DebugBuffer::new();
        debug_buf
            .format_debug_message_indexed(message1_str, values)
            .expect("Failed to format debug message");

        // Append second message
        debug_buf.write_str(message2_str).expect("Failed to append second message");

        let formatted_str = debug_buf.as_str();
        let total_len = formatted_str.len();

        let slice = arena.alloc_slice_from_fn(total_len, |i| formatted_str.as_bytes()[i])?;
        Ok(Self { slice })
    }

    /// Create formatted text using Display trait
    /// Replaces "{}" and "{:.N}" placeholders with values by index, with message2 appended after
    pub fn format_display<const LEN: usize, M1, M2, V>(
        arena: &mut Arena<LEN, I>,
        message1: M1,
        values: &[V],
        message2: M2,
    ) -> ArenaRes<Self>
    where
        M1: AsRef<str>,
        M2: AsRef<str>,
        V: core::fmt::Display,
    {
        let message1_str = message1.as_ref();
        let message2_str = message2.as_ref();
        debug_assert!(message1_str.is_ascii());
        debug_assert!(message2_str.is_ascii());

        // Validate format string first
        if let Err(err_msg) = Self::validate_format_string(message1_str) {
            panic!("Display format error: {} in: '{}'", err_msg, message1_str);
        }

        // Check placeholder count matches value count
        let placeholder_count = Self::count_placeholders(message1_str);
        if placeholder_count != values.len() {
            panic!(
                "Display format placeholder mismatch: found {} placeholders but {} values provided in: '{}'",
                placeholder_count,
                values.len(),
                message1_str
            );
        }

        // Format using display trait with indexed values
        let mut debug_buf = DebugBuffer::new();
        debug_buf
            .format_display_message_indexed(message1_str, values)
            .expect("Failed to format display message");

        // Append second message
        debug_buf.write_str(message2_str).expect("Failed to append second message");

        let formatted_str = debug_buf.as_str();
        let total_len = formatted_str.len();

        let slice = arena.alloc_slice_from_fn(total_len, |i| formatted_str.as_bytes()[i])?;
        Ok(Self { slice })
    }

    /// Create formatted text with full format support
    /// Supports format specifiers: "{}", "{:?}", "{:.N}"
    /// Requires both Debug and Display traits
    pub fn format<const LEN: usize, M1, M2, V>(
        arena: &mut Arena<LEN, I>,
        message1: M1,
        value: V,
        message2: M2,
    ) -> ArenaRes<Self>
    where
        M1: AsRef<str>,
        M2: AsRef<str>,
        V: core::fmt::Display + core::fmt::Debug,
    {
        let message1_str = message1.as_ref();
        let message2_str = message2.as_ref();
        debug_assert!(message1_str.is_ascii());
        debug_assert!(message2_str.is_ascii());

        // Format the complete message with value into a buffer
        let mut debug_buf = DebugBuffer::new();
        if debug_buf.format_message(message1_str, &value).is_err() {
            return Err(ArenaErr::InvalidBounds);
        }

        // Append second message
        if debug_buf.write_str(message2_str).is_err() {
            return Err(ArenaErr::InvalidBounds);
        }

        let formatted_str = debug_buf.as_str();
        let total_len = formatted_str.len();

        let slice = arena.alloc_slice_from_fn(total_len, |i| formatted_str.as_bytes()[i])?;
        Ok(Self { slice })
    }

    /// Count placeholders in a format string
    fn count_placeholders(message: &str) -> usize {
        let mut count = 0;
        let mut remaining = message;
        while let Some((_, end, _)) = crate::text::debug_buffer::parse_format_string(remaining) {
            count += 1;
            remaining = &remaining[end..];
        }
        count
    }

    /// Validate format string and provide clear error for invalid placeholders
    fn validate_format_string(message: &str) -> Result<(), &'static str> {
        let mut remaining = message;

        while let Some(start) = remaining.find('{') {
            if let Some(end_pos) = remaining[start..].find('}') {
                let placeholder = &remaining[start..start + end_pos + 1];

                // Try to parse this placeholder
                if crate::text::debug_buffer::parse_format_string(&remaining[start..]).is_none() {
                    // Check for common invalid patterns
                    if placeholder.contains("?}") && !placeholder.ends_with(":?}") {
                        return Err(
                            "Invalid format specifier: precision with debug (?), use either {:.N} or {:?}",
                        );
                    }
                    if placeholder.contains(":.") && placeholder.contains("?") {
                        return Err(
                            "Invalid format specifier: cannot combine precision and debug formatting",
                        );
                    }
                    // Check for shorthand precision format {:N}
                    if placeholder.starts_with("{:")
                        && placeholder.ends_with("}")
                        && placeholder.len() > 3
                    {
                        let inner = &placeholder[2..placeholder.len() - 1];
                        if inner.parse::<usize>().is_ok() {
                            return Err(
                                "Invalid format specifier: use {:.N} instead of {:N} for precision formatting",
                            );
                        }
                    }
                    return Err("Invalid format specifier: supported formats are {}, {:?}, {:.N}");
                }

                remaining = &remaining[start + end_pos + 1..];
            } else {
                return Err("Invalid format string: found '{' without matching '}'");
            }
        }

        // Check for unmatched '}'
        if remaining.contains('}') {
            return Err("Invalid format string: found '}' without matching '{'");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Arena;

    #[test]
    #[should_panic(expected = "use {:.N} instead of {:N}")]
    fn test_shorthand_precision_rejected() {
        let mut arena = Arena::<1024, u32>::new();

        // Test that {:0} shorthand is rejected with helpful message
        let values = [60.5, 30.0];
        let _result = Text::format_display(&mut arena, "fps: {:.1} / {:0}", &values, "");
    }

    #[test]
    fn test_zero_precision_works() {
        let mut arena = Arena::<1024, u32>::new();

        // Test that {:.0} works correctly
        let values = [3.14159, 2.99999];
        let result =
            Text::format_display(&mut arena, "pi: {:.0}, rounded: {:.0}", &values, "").unwrap();
        let formatted = result.as_str(&arena).unwrap();
        assert_eq!(formatted, "pi: 3, rounded: 3");
    }
}
