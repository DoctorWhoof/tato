use core::fmt::Write;

/// Simple fixed-size buffer for formatting Debug values
pub(crate) struct DebugBuffer {
    buf: [u8; 256],
    len: usize,
}

impl DebugBuffer {
    pub(crate) fn new() -> Self {
        Self { buf: [0; 256], len: 0 }
    }

    pub(crate) fn as_str(&self) -> &str {
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
