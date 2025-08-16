use super::*;

#[test]
fn test_text_from_str() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::from_str(&mut arena, "Hello, World!").unwrap();
    assert_eq!(text.len(), 13);

    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "Hello, World!");
}

#[test]
fn test_text_format_display() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::format(&mut arena, "value: {}", 42).unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "value: 42");
}

#[test]
fn test_text_format_debug() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::format(&mut arena, "debug: {:?}", "Hello").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "debug: \"Hello\"");
}

#[test]
fn test_text_format_precision() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::format(&mut arena, "pi: {:.2}", 3.14159).unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "pi: 3.14");
}

#[test]
fn test_text_format_fallback() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::format(&mut arena, "no format:", 42).unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "no format:42");
}

#[test]
fn test_text_format_string_display() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::format(&mut arena, "greeting: {}", "Hello").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "greeting: Hello"); // No quotes with Display
}

#[test]
fn test_text_format_debug_only() {
    let mut arena: Arena<1024> = Arena::new();

    // Test with a Debug-only type (tuples implement Debug but not Display)
    let tuple = (1, 2, 3);
    let text = Text::format_debug(&mut arena, "data: {:?}", &tuple).unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "data: (1, 2, 3)");
}

#[test]
fn test_text_format_debug_fallback() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::format_debug(&mut arena, "no format:", 42).unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "no format:42");
}

#[test]
fn test_text_format_display_only() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::format_display(&mut arena, "value: {}", 42).unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "value: 42");
}

#[test]
fn test_text_format_display_precision() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::format_display(&mut arena, "pi: {:.3}", 3.14159).unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "pi: 3.142");
}

#[test]
fn test_text_format_display_fallback() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::format_display(&mut arena, "no format:", 42).unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "no format:42");
}
