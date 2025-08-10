
#[doc(hidden)]
#[macro_export]
/// Simply formats text characters so that "Videochip Error:" prints in red, and "body" prints in orange.
macro_rules! err {
    ($body:expr) => {
        concat!("\x1b[31mVideochip Error: \x1b[33m", $body, "\x1b[0m")
    };
}

#[doc(hidden)]
#[macro_export]
/// Simply formats text characters so that "Videochip Error:" prints in red, and "body" prints in orange.
macro_rules! warn {
    ($body:expr) => {
        concat!("\x1b[31mVideochip Warning: \x1b[33m", $body, "\x1b[0m")
    };
}
