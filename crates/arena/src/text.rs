//! Text utilities for Pool<u8> - treating byte pools as strings

use crate::{Arena, ArenaIndex, Pool};
use core::fmt::{Debug, Write};

/// Type alias for text stored as a Pool<u8>
pub type Text<SizeType = u16, Marker = ()> = Pool<u8, SizeType, Marker>;

/// Simple fixed-size buffer for formatting Debug values
struct DebugBuffer {
    buf: [u8; 256],
    len: usize,
}

impl DebugBuffer {
    fn new() -> Self {
        Self {
            buf: [0; 256],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        // Safety: We only write valid UTF-8 via core::fmt::Write
        unsafe { core::str::from_utf8_unchecked(&self.buf[..self.len]) }
    }
}

impl Write for DebugBuffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        if self.len + bytes.len() > self.buf.len() {
            return Err(core::fmt::Error);
        }
        self.buf[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }
}

/// Implementation for "Pool<u8>"" specifically to add text functionality. You can convert to and from
/// &str, and use [Text::format] for very basic message+value formatting in no_std environments.
impl<SizeType, Marker> Text<SizeType, Marker>
where
    SizeType: ArenaIndex,
{
    /// Get the text as &str (requires arena for safety)
    /// Returns None if the bytes are not valid UTF-8
    pub fn as_str<'a, const LEN: usize>(&self, arena: &'a Arena<LEN, SizeType, Marker>) -> Option<&'a str> {
            let bytes = arena.get_pool(self)?;
            core::str::from_utf8(bytes).ok()
        }

    /// Create text from a string slice
    pub fn from_str<const LEN: usize>(
        arena: &mut Arena<LEN, SizeType, Marker>,
        s: &str,
    ) -> Option<Self> {
        arena.alloc_pool_from_fn(s.len(), |i| s.as_bytes()[i])
    }

    /// Create formatted text from message and value
    pub fn format<const LEN: usize, M, V>(
        arena: &mut Arena<LEN, SizeType, Marker>,
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
        let total_len = message_str.len() + value_str.len();
        arena.alloc_pool_from_fn(total_len, |i| {
            if i < message_str.len() {
                message_str.as_bytes()[i]
            } else {
                value_str.as_bytes()[i - message_str.len()]
            }
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_from_str() {
        let mut arena: Arena<1024> = Arena::new();

        let text = Pool::from_str(&mut arena, "Hello, World!").unwrap();
        assert_eq!(text.len(), 13);

        let s = text.as_str(&arena).unwrap();
        assert_eq!(s, "Hello, World!");
    }

    #[test]
    fn test_text_format() {
        let mut arena: Arena<1024> = Arena::new();

        let text = Pool::format(&mut arena, "greeting", "Hello").unwrap();
        let s = text.as_str(&arena).unwrap();
        assert_eq!(s, "greeting: Hello");
    }
}
