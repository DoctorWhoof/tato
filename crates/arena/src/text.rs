//! Text utilities for Slice<u8> - treating byte slices as strings
mod debug_buffer;
use debug_buffer::*;

use crate::{Arena, ArenaIndex, ArenaResult, ArenaError, Buffer};
use core::fmt::Write;

/// Type alias for text stored as a Slice<u8>
pub type Text<Idx = u16> = Buffer<u8, Idx>;

/// Implementation for "Slice<u8>"" specifically to add text functionality. You can convert to and from
/// &str, and use [Text::format] for very basic message+value formatting in no_std environments.
impl<Idx> Text<Idx>
where
    Idx: ArenaIndex,
{
    /// Get the text as &str (requires arena for safety)
    /// Returns None if the bytes are not valid UTF-8
    // TODO: Return result intead of option
    pub fn as_str<'a, const LEN: usize>(&self, arena: &'a Arena<LEN, Idx>) -> Option<&'a str> {
        let bytes = arena.get_slice(&self.slice).ok()?;
        core::str::from_utf8(bytes).ok()
    }

    /// Create text from a string slice
    pub fn from_str<const LEN: usize>(arena: &mut Arena<LEN, Idx>, s: &str) -> ArenaResult<Self> {
        let bytes = s.as_bytes();
        let len = Idx::from_usize_checked(s.len()).ok_or(ArenaError::IndexConversion)?;
        Buffer::from_fn(arena, len, |i| bytes[i])
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

    /// Create formatted text using Debug trait
    /// Replaces "{:?}" placeholders with values by index, with message2 appended after
    pub fn format_dbg<const LEN: usize, M1, M2, V>(
        arena: &mut Arena<LEN, Idx>,
        message1: M1,
        values: &[V],
        message2: M2,
    ) -> ArenaResult<Self>
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
        let total_len = Idx::from_usize_checked(formatted_str.len()).ok_or(ArenaError::IndexConversion)?;

        Buffer::from_fn(arena, total_len, |i| formatted_str.as_bytes()[i])
    }

    /// Create formatted text using Display trait
    /// Replaces "{}" and "{:.N}" placeholders with values by index, with message2 appended after
    pub fn format_display<const LEN: usize, M1, M2, V>(
        arena: &mut Arena<LEN, Idx>,
        message1: M1,
        values: &[V],
        message2: M2,
    ) -> ArenaResult<Self>
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
        let total_len = Idx::from_usize_checked(formatted_str.len()).ok_or(ArenaError::IndexConversion)?;

        Buffer::from_fn(arena, total_len, |i| formatted_str.as_bytes()[i])
    }

    /// Create formatted text with full format support
    /// Supports format specifiers: "{}", "{:?}", "{:.N}"
    /// Requires both Debug and Display traits
    pub fn format<const LEN: usize, M1, M2, V>(
        arena: &mut Arena<LEN, Idx>,
        message1: M1,
        value: V,
        message2: M2,
    ) -> ArenaResult<Self>
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
            return Err(ArenaError::InvalidBounds);
        }

        // Append second message
        if debug_buf.write_str(message2_str).is_err() {
            return Err(ArenaError::InvalidBounds);
        }

        let formatted_str = debug_buf.as_str();
        let total_len = Idx::from_usize_checked(formatted_str.len()).ok_or(ArenaError::IndexConversion)?;

        Buffer::from_fn(arena, total_len, |i| formatted_str.as_bytes()[i])
    }

    // TODO: Delete this. A buffer of arena Ids is much more memory friendly,
    // since it doesn't pre-allocate all items. Still in use in Tato.debug_strings
    /// A Buffer of Text lines (which are, themselves, buffers).
    /// Helps to get around borrowing issues since the buffer and the text lines
    /// are in the same arena.
    /// Warning: All text lines are pre-initialized!
    /// Use clear() after creating if you want to start empty.
    pub fn text_multi_buffer<const ARENA_LEN: usize>(
        arena: &mut Arena<ARENA_LEN, Idx>,
        item_count: Idx,
        item_length: Idx,
        init_with_zero_len: bool,
    ) -> ArenaResult<Buffer<Text<Idx>, Idx>> {
        let mut result = Self::multi_buffer::<ARENA_LEN>(arena, item_count, item_length)?;
        if init_with_zero_len {
            result.clear();
        }
        Ok(result)
    }
}
