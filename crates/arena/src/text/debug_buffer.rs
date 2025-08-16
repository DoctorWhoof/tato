use core::fmt::{Write, Display, Debug, Formatter};

/// Format specifier parsed from format strings
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FormatSpec {
    Display,
    Debug,
    DisplayWithPrecision(usize),
}

/// Parse a simple format string to extract format specifier
/// Supports: "{}", "{:?}", "{:.N}"
/// Returns (start_pos, end_pos, format_spec) of the first format specifier found
pub(crate) fn parse_format_string(s: &str) -> Option<(usize, usize, FormatSpec)> {
    let start = s.find('{')?;
    let end = s.find('}')? + 1;

    let spec_str = &s[start+1..end-1];

    let spec = if spec_str.is_empty() {
        FormatSpec::Display
    } else if spec_str == ":?" {
        FormatSpec::Debug
    } else if spec_str.starts_with(":.") {
        let precision_str = &spec_str[2..];
        let precision = precision_str.parse().ok()?;
        FormatSpec::DisplayWithPrecision(precision)
    } else {
        return None; // Unsupported format
    };

    Some((start, end, spec))
}

/// Simple fixed-size buffer for formatting values
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

    /// Format a value according to the format specifier
    pub(crate) fn format_value<T>(&mut self, value: &T, spec: FormatSpec) -> Result<(), core::fmt::Error>
    where
        T: Debug + Display,
    {
        match spec {
            FormatSpec::Display => write!(self, "{}", value),
            FormatSpec::Debug => write!(self, "{:?}", value),
            FormatSpec::DisplayWithPrecision(precision) => {
                self.format_with_precision(value, precision)
            }
        }
    }

    /// Format a value with specific precision (for numeric types)
    fn format_with_precision<T>(&mut self, value: &T, precision: usize) -> Result<(), core::fmt::Error>
    where
        T: Display,
    {
        // Create a wrapper that formats with the specified precision
        struct PrecisionWrapper<'a, T> {
            value: &'a T,
            precision: usize,
        }

        impl<T: Display> Display for PrecisionWrapper<'_, T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:.1$}", self.value, self.precision)
            }
        }

        let wrapper = PrecisionWrapper { value, precision };
        write!(self, "{}", wrapper)
    }

    /// Format a complete message with a single format specifier and value
    /// Replaces the format specifier in the message with the formatted value
    pub(crate) fn format_message<T>(
        &mut self,
        message: &str,
        value: &T,
    ) -> Result<(), core::fmt::Error>
    where
        T: Display + Debug,
    {
        if let Some((start, end, spec)) = parse_format_string(message) {
            // Write the part before the format specifier
            self.write_str(&message[..start])?;

            // Format and write the value
            self.format_value(value, spec)?;

            // Write the part after the format specifier
            self.write_str(&message[end..])?;

            Ok(())
        } else {
            // No format specifier found
            self.write_str(message)?;
            write!(self, "{:?}", value)
        }
    }

    /// Format a message using only Debug trait - supports {:?} and fallback
    pub(crate) fn format_debug_message<T>(
        &mut self,
        message: &str,
        value: &T,
    ) -> Result<(), core::fmt::Error>
    where
        T: Debug,
    {
        if let Some((start, end, spec)) = parse_format_string(message) {
            match spec {
                FormatSpec::Debug => {
                    self.write_str(&message[..start])?;
                    write!(self, "{:?}", value)?;
                    self.write_str(&message[end..])?;
                    Ok(())
                }
                _ => {
                    // Unsupported format for Debug-only, fall back to debug with separator
                    self.write_str(message)?;
                    write!(self, "{:?}", value)
                }
            }
        } else {
            // No format specifier found
            self.write_str(message)?;
            write!(self, "{:?}", value)
        }
    }

    /// Format a message using only Display trait - supports {} and {:.N} and fallback
    pub(crate) fn format_display_message<T>(
        &mut self,
        message: &str,
        value: &T,
    ) -> Result<(), core::fmt::Error>
    where
        T: Display,
    {
        if let Some((start, end, spec)) = parse_format_string(message) {
            match spec {
                FormatSpec::Display => {
                    self.write_str(&message[..start])?;
                    write!(self, "{}", value)?;
                    self.write_str(&message[end..])?;
                    Ok(())
                }
                FormatSpec::DisplayWithPrecision(precision) => {
                    self.write_str(&message[..start])?;
                    self.format_with_precision(value, precision)?;
                    self.write_str(&message[end..])?;
                    Ok(())
                }
                FormatSpec::Debug => {
                    // Debug format not supported for Display-only, fall back
                    self.write_str(message)?;
                    write!(self, "{}", value)
                }
            }
        } else {
            // No format specifier found
            self.write_str(message)?;
            write!(self, "{}", value)
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_format_string() {
        assert_eq!(parse_format_string("hello {}"), Some((6, 8, FormatSpec::Display)));
        assert_eq!(parse_format_string("debug {:?}"), Some((6, 10, FormatSpec::Debug)));
        assert_eq!(parse_format_string("precise {:.2}"), Some((8, 13, FormatSpec::DisplayWithPrecision(2))));
        assert_eq!(parse_format_string("no format"), None);
    }

    #[test]
    fn test_format_message() {
        let mut buf = DebugBuffer::new();
        buf.format_message("value: {}", &42).unwrap();
        assert_eq!(buf.as_str(), "value: 42");

        let mut buf = DebugBuffer::new();
        buf.format_message("debug: {:?}", &"hello").unwrap();
        assert_eq!(buf.as_str(), "debug: \"hello\"");

        let mut buf = DebugBuffer::new();
        buf.format_message("pi: {:.2}", &3.14159).unwrap();
        assert_eq!(buf.as_str(), "pi: 3.14");

        let mut buf = DebugBuffer::new();
        buf.format_message("fallback: ", &42).unwrap();
        assert_eq!(buf.as_str(), "fallback: 42");
    }
}
