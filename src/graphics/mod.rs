mod text;
pub use text::*;

mod maps;

// "map" functions are implemented as methods, and are already exported
// together with the Tato struct. No need to make them public again from here.
// pub use maps::*;
