//! Text utilities for Slice<u8> - treating byte slices as strings
mod debug_buffer;
use debug_buffer::*;

use crate::{Arena, ArenaIndex, Buffer};

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
    pub fn as_str<'a, const LEN: usize>(&self, arena: &'a Arena<LEN, Idx>) -> Option<&'a str> {
        let bytes = arena.get_slice(&self.slice)?;
        core::str::from_utf8(bytes).ok()
    }

    /// Create text from a string slice
    pub fn from_str<const LEN: usize>(arena: &mut Arena<LEN, Idx>, s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        let len = Idx::from_usize_checked(s.len()).unwrap();
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
                        return Err("Invalid format specifier: precision with debug (?), use either {:.N} or {:?}");
                    }
                    if placeholder.contains(":.") && placeholder.contains("?") {
                        return Err("Invalid format specifier: cannot combine precision and debug formatting");
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
    /// Replaces "{:?}" placeholders with values by index
    pub fn format_dbg<const LEN: usize, M, V>(
        arena: &mut Arena<LEN, Idx>,
        message: M,
        values: &[V],
    ) -> Option<Self>
    where
        M: AsRef<str>,
        V: core::fmt::Debug,
    {
        let message_str = message.as_ref();
        debug_assert!(message_str.is_ascii());

        // Validate format string first
        if let Err(err_msg) = Self::validate_format_string(message_str) {
            panic!("Debug format error: {} in: '{}'", err_msg, message_str);
        }

        // Check placeholder count matches value count
        let placeholder_count = Self::count_placeholders(message_str);
        if placeholder_count != values.len() {
            panic!("Debug format placeholder mismatch: found {} placeholders but {} values provided in: '{}'",
                   placeholder_count, values.len(), message_str);
        }

        // Format using debug trait with indexed values
        let mut debug_buf = DebugBuffer::new();
        debug_buf.format_debug_message_indexed(message_str, values)
            .expect("Failed to format debug message");

        let formatted_str = debug_buf.as_str();
        let total_len = Idx::from_usize_checked(formatted_str.len())?;

        Buffer::from_fn(arena, total_len, |i| formatted_str.as_bytes()[i])
    }

    /// Create formatted text using Display trait
    /// Replaces "{}" and "{:.N}" placeholders with values by index
    pub fn format_display<const LEN: usize, M, V>(
        arena: &mut Arena<LEN, Idx>,
        message: M,
        values: &[V],
    ) -> Option<Self>
    where
        M: AsRef<str>,
        V: core::fmt::Display,
    {
        let message_str = message.as_ref();
        debug_assert!(message_str.is_ascii());

        // Validate format string first
        if let Err(err_msg) = Self::validate_format_string(message_str) {
            panic!("Display format error: {} in: '{}'", err_msg, message_str);
        }

        // Check placeholder count matches value count
        let placeholder_count = Self::count_placeholders(message_str);
        if placeholder_count != values.len() {
            panic!("Display format placeholder mismatch: found {} placeholders but {} values provided in: '{}'",
                   placeholder_count, values.len(), message_str);
        }

        // Format using display trait with indexed values
        let mut debug_buf = DebugBuffer::new();
        debug_buf.format_display_message_indexed(message_str, values)
            .expect("Failed to format display message");

        let formatted_str = debug_buf.as_str();
        let total_len = Idx::from_usize_checked(formatted_str.len())?;

        Buffer::from_fn(arena, total_len, |i| formatted_str.as_bytes()[i])
    }

    /// Create formatted text with full format support
    /// Supports format specifiers: "{}", "{:?}", "{:.N}"
    /// Requires both Debug and Display traits
    pub fn format<const LEN: usize, M, V>(
        arena: &mut Arena<LEN, Idx>,
        message: M,
        value: V,
    ) -> Option<Self>
    where
        M: AsRef<str>,
        V: core::fmt::Display + core::fmt::Debug,
    {
        let message_str = message.as_ref();
        debug_assert!(message_str.is_ascii());

        // Format the complete message with value into a buffer
        let mut debug_buf = DebugBuffer::new();
        if debug_buf.format_message(message_str, &value).is_err() {
            return None;
        }

        let formatted_str = debug_buf.as_str();
        let total_len = Idx::from_usize_checked(formatted_str.len())?;

        Buffer::from_fn(arena, total_len, |i| formatted_str.as_bytes()[i])
    }

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
    ) -> Option<Buffer<Text<Idx>, Idx>> {
        let mut result = Self::multi_buffer::<ARENA_LEN>(arena, item_count, item_length)?;
        if init_with_zero_len {
            result.clear();
        }
        Some(result)
    }
}
