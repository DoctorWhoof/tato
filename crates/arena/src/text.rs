//! Text utilities for Slice<u8> - treating byte slices as strings
mod debug_buffer;
use debug_buffer::*;

use crate::{ArenaErr, ArenaIndex, ArenaOps, ArenaRes, Buffer, Slice};
use core::fmt::Write;

/// Text stored as bytes in the arena
#[derive(Debug, Clone, Copy)]
pub struct Text<I = u32, M = ()> {
    pub slice: Slice<u8, I, M>,
}

/// Unnalocated Text (will fail any attempt to obtain its data from an arena)
impl<I, M> Default for Text<I, M>
where
    I: ArenaIndex,
{
    fn default() -> Self {
        Self { slice: Default::default() }
    }
}

/// Implementation for "Slice<u8>"" specifically to add text functionality. You can convert to and from
/// &str, and use [Text::format] for very basic message+value formatting in no_std environments.
impl<I, M> Text<I, M>
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
    pub fn as_slice<'a, A>(&self, arena: &'a A) -> ArenaRes<&'a [u8]>
    where
        A: ArenaOps<I, M>,
    {
        arena.get_slice(self.slice.clone())
    }

    /// Get the text as &str (requires arena for safety)
    /// Returns an Error if the bytes are not valid UTF-8
    pub fn as_str<'a, A>(&self, arena: &'a A) -> ArenaRes<&'a str>
    where
        A: ArenaOps<I, M>,
    {
        let bytes = arena.get_slice(self.slice.clone())?;
        if let Ok(value) = core::str::from_utf8(bytes) {
            Ok(value)
        } else {
            Err(ArenaErr::InvalidUTF8)
        }
    }

    /// Create text from a string slice
    pub fn from_str<A>(arena: &mut A, s: &str) -> ArenaRes<Self>
    where
        A: ArenaOps<I, M>,
    {
        let bytes = s.as_bytes();
        let len = s.len();
        let slice = arena.alloc_slice_from_fn(len, |i| bytes[i])?;
        Ok(Self { slice })
    }

    /// Create text from a Buffer<u8>
    pub fn from_buffer<A>(
        arena: &mut A,
        buffer: &Buffer<u8, I, M>,
    ) -> ArenaRes<Self>
    where
        A: ArenaOps<I, M>,
    {
        let used_len_usize = buffer.len();

        // Save tail position for restoration
        let saved_tail = arena.save_tail_position();

        // Allocate temporary space from tail
        let temp_ptr = match arena.tail_alloc_bytes_internal(used_len_usize, 1) {
            Ok(ptr) => ptr,
            Err(e) => {
                arena.restore_tail_position(saved_tail);
                return Err(e);
            }
        };

        // Copy buffer contents to tail space
        unsafe {
            let slice_content = match arena.get_slice(buffer.slice.clone()) {
                Ok(content) => content,
                Err(e) => {
                    arena.restore_tail_position(saved_tail);
                    return Err(e);
                }
            };
            core::ptr::copy_nonoverlapping(slice_content[..used_len_usize].as_ptr(), temp_ptr, used_len_usize);
        }

        // Allocate permanent slice from the temp data
        let slice = arena.alloc_slice_from_fn(used_len_usize, |i| unsafe {
            *temp_ptr.add(i)
        });

        // Restore tail position (free temp space)
        arena.restore_tail_position(saved_tail);

        slice.map(|s| Self { slice: s })
    }

    /// Create text from a valid (but non-zero) ASCII slice
    pub fn from_bytes<A>(
        arena: &mut A,
        bytes: &[u8],
    ) -> ArenaRes<Self>
    where
        A: ArenaOps<I, M>,
    {
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
    pub fn from_fn<A, F>(
        arena: &mut A,
        length: usize,
        func: F,
    ) -> ArenaRes<Self>
    where
        A: ArenaOps<I, M>,
        F: Fn(usize) -> u8,
    {
        let slice = arena.alloc_slice_from_fn(length, func)?;
        Ok(Self { slice })
    }

    /// Join multiple text instances into a single text.
    pub fn join<A>(
        arena: &mut A,
        sources: &[Text<I, M>],
    ) -> ArenaRes<Self>
    where
        A: ArenaOps<I, M>,
    {
        if sources.is_empty() {
            return Ok(Self::default());
        }

        // Calculate total length first
        let mut total_len = 0usize;
        for text in sources {
            total_len += text.slice.len().to_usize();
        }
        let final_len = total_len;

        // Save tail position
        let saved_tail = arena.save_tail_position();

        // Allocate temp space from tail
        let temp_ptr = match arena.tail_alloc_bytes_internal(final_len, 1) {
            Ok(ptr) => ptr,
            Err(e) => {
                arena.restore_tail_position(saved_tail);
                return Err(e);
            }
        };

        // Copy all source texts into the temp space
        let mut write_offset = 0usize;
        for text in sources {
            let text_len = text.slice.len().to_usize();
            match arena.get_slice(text.slice.clone()) {
                Ok(bytes) => unsafe {
                    core::ptr::copy_nonoverlapping(
                        bytes.as_ptr(),
                        temp_ptr.add(write_offset),
                        text_len
                    );
                },
                Err(e) => {
                    arena.restore_tail_position(saved_tail);
                    return Err(e);
                }
            }
            write_offset += text_len;
        }

        // Allocate permanent slice from the temp data
        let slice = arena.alloc_slice_from_fn(final_len, |i| unsafe {
            *temp_ptr.add(i)
        });

        // Restore tail position (free temp space)
        arena.restore_tail_position(saved_tail);

        slice.map(|s| Self { slice: s })
    }

    /// Join multiple byte slices into a single text.
    pub fn join_bytes<A>(
        arena: &mut A,
        slices: &[&[u8]],
    ) -> ArenaRes<Self>
    where
        A: ArenaOps<I, M>,
    {
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

    /// Internal helper for formatting using tail allocation
    #[doc(hidden)]
    fn format_with_tail<A, F>(
        arena: &mut A,
        estimate_size: usize,
        format_fn: F,
    ) -> ArenaRes<Self>
    where
        A: ArenaOps<I, M>,
        F: FnOnce(&mut [u8]) -> Result<usize, ArenaErr>,
    {
        // Save tail position
        let saved_tail = arena.save_tail_position();

        // Allocate temp space from tail (use estimate or reasonable default)
        let alloc_size = if estimate_size > 0 { estimate_size } else { 256 };
        let temp_ptr = match arena.tail_alloc_bytes_internal(alloc_size, 1) {
            Ok(ptr) => ptr,
            Err(e) => {
                arena.restore_tail_position(saved_tail);
                return Err(e);
            }
        };

        // Format into the tail space
        let actual_len = unsafe {
            let temp_slice = core::slice::from_raw_parts_mut(temp_ptr, alloc_size);
            match format_fn(temp_slice) {
                Ok(len) => len,
                Err(e) => {
                    arena.restore_tail_position(saved_tail);
                    return Err(e);
                }
            }
        };

        // Allocate permanent slice from the formatted data
        let slice = arena.alloc_slice_from_fn(actual_len, |i| unsafe {
            *temp_ptr.add(i)
        });

        // Restore tail position (free temp space)
        arena.restore_tail_position(saved_tail);

        slice.map(|s| Self { slice: s })
    }

    /// Create formatted text using Debug trait
    /// Replaces "{:?}" placeholders with values by index, with message2 appended after
    pub fn format_dbg<A, M1, M2, V>(
        arena: &mut A,
        message1: M1,
        values: &[V],
        message2: M2,
    ) -> ArenaRes<Self>
    where
        A: ArenaOps<I, M>,
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

        // Estimate size needed
        let estimate = message1_str.len() + message2_str.len() + (values.len() * 20);

        // Format using tail allocation
        Self::format_with_tail(arena, estimate, |temp_buf| {
            let mut debug_buf = DebugBuffer::new();
            debug_buf
                .format_debug_message_indexed(message1_str, values)
                .map_err(|_| ArenaErr::FormatError)?;
            debug_buf.write_str(message2_str)
                .map_err(|_| ArenaErr::FormatError)?;

            let formatted_str = debug_buf.as_str();
            let len = formatted_str.len();
            if len > temp_buf.len() {
                return Err(ArenaErr::OutOfSpace {
                    requested: len,
                    available: temp_buf.len(),
                });
            }
            temp_buf[..len].copy_from_slice(formatted_str.as_bytes());
            Ok(len)
        })
    }

    /// Create formatted text using Display trait
    /// Replaces "{}" and "{:.N}" placeholders with values by index, with message2 appended after
    pub fn format_display<A, M1, M2, V>(
        arena: &mut A,
        message1: M1,
        values: &[V],
        message2: M2,
    ) -> ArenaRes<Self>
    where
        A: ArenaOps<I, M>,
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

        // Estimate size needed
        let estimate = message1_str.len() + message2_str.len() + (values.len() * 20);

        // Format using tail allocation
        Self::format_with_tail(arena, estimate, |temp_buf| {
            let mut debug_buf = DebugBuffer::new();
            debug_buf
                .format_display_message_indexed(message1_str, values)
                .map_err(|_| ArenaErr::FormatError)?;
            debug_buf.write_str(message2_str)
                .map_err(|_| ArenaErr::FormatError)?;

            let formatted_str = debug_buf.as_str();
            let len = formatted_str.len();
            if len > temp_buf.len() {
                return Err(ArenaErr::OutOfSpace {
                    requested: len,
                    available: temp_buf.len()
                });
            }
            temp_buf[..len].copy_from_slice(formatted_str.as_bytes());
            Ok(len)
        })
    }

    /// Create formatted text with full format support
    /// Supports format specifiers: "{}", "{:?}", "{:.N}"
    /// Requires both Debug and Display traits
    pub fn format<A, M1, M2, V>(
        arena: &mut A,
        message1: M1,
        value: V,
        message2: M2,
    ) -> ArenaRes<Self>
    where
        A: ArenaOps<I, M>,
        M1: AsRef<str>,
        M2: AsRef<str>,
        V: core::fmt::Display + core::fmt::Debug,
    {
        let message1_str = message1.as_ref();
        let message2_str = message2.as_ref();
        debug_assert!(message1_str.is_ascii());
        debug_assert!(message2_str.is_ascii());

        // Estimate size needed
        let estimate = message1_str.len() + message2_str.len() + 20;

        // Format using tail allocation
        Self::format_with_tail(arena, estimate, |temp_buf| {
            let mut debug_buf = DebugBuffer::new();
            debug_buf.format_message(message1_str, &value)
                .map_err(|_| ArenaErr::FormatError)?;
            debug_buf.write_str(message2_str)
                .map_err(|_| ArenaErr::FormatError)?;

            let formatted_str = debug_buf.as_str();
            let len = formatted_str.len();
            if len > temp_buf.len() {
                return Err(ArenaErr::OutOfSpace {
                    requested: len,
                    available: temp_buf.len()
                });
            }
            temp_buf[..len].copy_from_slice(formatted_str.as_bytes());
            Ok(len)
        })
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
