//! Text utilities for Slice<u8> - treating byte pools as strings
mod debug_buffer;
use debug_buffer::*;

use crate::{Arena, ArenaIndex, Buffer};
use core::fmt::{Write, Debug};
use std::fmt::Display;

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
        let bytes = arena.get_pool(&self.pool)?;
        core::str::from_utf8(bytes).ok()
    }

    /// Create text from a string slice
    pub fn from_str<const LEN: usize>(arena: &mut Arena<LEN, Idx>, s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        let len = Idx::from_usize_checked(s.len()).unwrap();
        Buffer::from_fn(arena, len, |i| bytes[i])
    }

    /// Create formatted text from message and value
    pub fn format<const LEN: usize, M, V>(
        arena: &mut Arena<LEN, Idx>,
        message: M,
        value: V,
    ) -> Option<Self>
    where
        M: AsRef<str>,
        V: Debug,
    {
        let message_str = message.as_ref();
        debug_assert!(message_str.is_ascii());

        // Format the debug value into a buffer
        let mut debug_buf = DebugBuffer::new();
        let value_str = if write!(&mut debug_buf, "{:?}", value).is_ok() {
            debug_buf.as_str()
        } else {
            "[DEBUG_TOO_LARGE]"
        };

        debug_assert!(value_str.is_ascii());

        // Calculate total length: message + ": " + value
        let total_len = Idx::from_usize_checked(message_str.len() + value_str.len()).unwrap();
        Buffer::from_fn(arena, total_len, |i| {
            if i < message_str.len() {
                message_str.as_bytes()[i]
            } else {
                value_str.as_bytes()[i - message_str.len()]
            }
        })
    }

    /// A Buffer of Text lines (which are, themselves, buffers).
    /// Helps to get around borrowing issues since the buffer and the text lines
    /// are in the same arena.
    pub fn text_buffer<const LEN: usize, const ARENA_LEN: usize>(
        arena: &mut Arena<ARENA_LEN, Idx>,
        item_length: Idx,
    ) -> Option<Buffer<Text<Idx>, Idx>> {
        let item_count: Idx = Idx::from_usize_checked(LEN)?;
        Self::multi_buffer::<LEN, ARENA_LEN, _>(arena, item_count, item_length, |_| ' ' as u8)
    }
}
