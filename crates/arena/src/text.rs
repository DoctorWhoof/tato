//! Text utilities for Slice<u8> - treating byte slices as strings
mod debug_buffer;
use debug_buffer::*;

use crate::{Arena, ArenaError, ArenaIndex, ArenaResult, Buffer, Slice};
use core::fmt::Write;

/// Text stored as bytes in the arena
#[derive(Debug, Clone)]
pub struct Text<Idx = u32> {
    pub slice: Slice<u8, Idx>,
}

/// Unnalocated Text (will fail any attempt to obtain its data from an arena)
impl<Idx> Default for Text<Idx>
where
    Idx: ArenaIndex,
{
    fn default() -> Self {
        Self { slice: Default::default() }
    }
}

/// Implementation for "Slice<u8>"" specifically to add text functionality. You can convert to and from
/// &str, and use [Text::format] for very basic message+value formatting in no_std environments.
impl<Idx> Text<Idx>
where
    Idx: ArenaIndex,
{
    /// Get the length of the text in bytes
    pub fn len(&self) -> usize {
        self.slice.len().to_usize()
    }

    /// Check if the text is empty
    pub fn is_empty(&self) -> bool {
        self.slice.len() == Idx::zero()
    }

    /// Get the text directly as a u8 slice
    // TODO: Return result intead of option
    pub fn as_slice<'a, const LEN: usize>(&self, arena: &'a Arena<LEN, Idx>) -> Option<&'a [u8]> {
        arena.get_slice(&self.slice).ok()
    }

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
        let slice = arena.alloc_slice_from_fn(len, |i| bytes[i])?;
        Ok(Self { slice })
    }

    /// Create text from a Buffer<u8>
    pub fn from_buffer<const LEN: usize>(
        arena: &mut Arena<LEN, Idx>,
        buffer: &Buffer<u8, Idx>,
    ) -> ArenaResult<Self> {
        let used_len = Idx::from_usize_checked(buffer.len()).ok_or(ArenaError::IndexConversion)?;
        let slice = arena.copy_slice_via_tail(&buffer.slice, used_len)?;
        Ok(Self { slice })
    }

    /// Create text from a valid (but non-zero) ASCII slice
    pub fn from_bytes<const LEN: usize>(
        arena: &mut Arena<LEN, Idx>,
        bytes: &[u8],
    ) -> ArenaResult<Self> {
        let mut len = Idx::zero();
        for i in 0..bytes.len() {
            let value = bytes[i];
            if value.is_ascii() && value > 0 {
                len = Idx::from_usize_checked(i + 1).unwrap()
            } else {
                break;
            }
        }
        if len > Idx::zero() {
            let slice = arena.alloc_slice_from_fn(len, |i| bytes[i])?;
            Ok(Self { slice })
        } else {
            ArenaResult::Err(ArenaError::InvalidOrEmptyUTF8)
        }
    }

    /// Create text from a function that generates bytes
    pub fn from_fn<const LEN: usize, F>(
        arena: &mut Arena<LEN, Idx>,
        length: Idx,
        func: F,
    ) -> ArenaResult<Self>
    where
        F: FnMut(usize) -> u8,
    {
        let slice = arena.alloc_slice_from_fn(length, func)?;
        Ok(Self { slice })
    }

    /// Join multiple text instances into a single text. Will fail for lengths over 1Kb.
    pub fn join<const LEN: usize>(
        arena: &mut Arena<LEN, Idx>,
        sources: &[Text<Idx>],
    ) -> ArenaResult<Self> {
        if sources.is_empty() {
            return Ok(Self::default());
        }

        // Calculate total length first
        let mut total_len = 0usize;
        for text in sources {
            total_len += text.slice.len().to_usize();
        }
        let final_len = Idx::from_usize_checked(total_len).ok_or(ArenaError::IndexConversion)?;

        // Use a temp arena to collect all the bytes, then copy to dest arena
        let mut temp_arena = Arena::<1024, Idx>::new();
        let temp_slice = temp_arena.alloc_slice_from_fn(final_len, |i| {
            // Find which source this byte belongs to
            let mut offset = 0usize;
            for text in sources {
                let text_len = text.slice.len().to_usize();
                if i < offset + text_len {
                    // Get the byte from this text
                    let bytes = arena.get_slice(&text.slice).unwrap();
                    return bytes[i - offset];
                }
                offset += text_len;
            }
            0 // Should never reach here
        })?;

        // Now copy from temp arena to dest arena
        let temp_bytes = temp_arena.get_slice(&temp_slice)?;
        let slice = arena.alloc_slice_from_fn(final_len, |i| temp_bytes[i])?;
        Ok(Self { slice })
    }

    /// Join multiple byte slices into a single text.
    pub fn join_bytes<const LEN: usize>(
        arena: &mut Arena<LEN, Idx>,
        slices: &[&[u8]],
    ) -> ArenaResult<Self> {
        if slices.is_empty() {
            let empty_slice = arena.alloc_slice::<u8>(Idx::zero())?;
            return Ok(Self { slice: empty_slice });
        }

        // Calculate total length first
        let mut total_len = 0usize;
        for slice in slices {
            total_len += slice.len();
        }
        let final_len = Idx::from_usize_checked(total_len).ok_or(ArenaError::IndexConversion)?;

        // Allocate and fill the result slice
        let result_slice = arena.alloc_slice_from_fn(final_len, |i| {
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

        Ok(Self { slice: result_slice })
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
        let total_len =
            Idx::from_usize_checked(formatted_str.len()).ok_or(ArenaError::IndexConversion)?;

        let slice = arena.alloc_slice_from_fn(total_len, |i| formatted_str.as_bytes()[i])?;
        Ok(Self { slice })
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
        let total_len =
            Idx::from_usize_checked(formatted_str.len()).ok_or(ArenaError::IndexConversion)?;

        let slice = arena.alloc_slice_from_fn(total_len, |i| formatted_str.as_bytes()[i])?;
        Ok(Self { slice })
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
        let total_len =
            Idx::from_usize_checked(formatted_str.len()).ok_or(ArenaError::IndexConversion)?;

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
