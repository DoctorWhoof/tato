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
fn test_text_format_display_single() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::format(&mut arena, "value: {}", 42, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "value: 42");
}

#[test]
fn test_text_format_debug_single() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::format(&mut arena, "debug: {:?}", "Hello", "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "debug: \"Hello\"");
}

#[test]
fn test_text_format_display_indexed() {
    let mut arena: Arena<1024> = Arena::new();

    let values = [42, 100];
    let text = Text::format_display(&mut arena, "first: {}, second: {}", &values, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "first: 42, second: 100");
}

#[test]
fn test_text_format_debug_indexed() {
    let mut arena: Arena<1024> = Arena::new();

    let values = ["hello", "world"];
    let text = Text::format_dbg(&mut arena, "first: {:?}, second: {:?}", &values, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "first: \"hello\", second: \"world\"");
}

#[test]
fn test_text_format_display_precision() {
    let mut arena: Arena<1024> = Arena::new();

    let values = [3.14159, 2.71828];
    let text = Text::format_display(&mut arena, "pi: {:.2}, e: {:.3}", &values, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "pi: 3.14, e: 2.718");
}



#[test]
fn test_text_format_empty_values() {
    let mut arena: Arena<1024> = Arena::new();

    let values: &[i32] = &[];
    let text = Text::format_display(&mut arena, "no placeholders", values, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "no placeholders");
}

#[test]
fn test_text_format_debug_with_display_specifier() {
    let mut arena: Arena<1024> = Arena::new();

    // format_dbg should now accept {} specifiers but still use Debug formatting
    let values = ["hello", "world"];
    let text = Text::format_dbg(&mut arena, "first: {}, second: {}", &values, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "first: \"hello\", second: \"world\"");

    // Test with precision specifier too  
    let values = [42];
    let text = Text::format_dbg(&mut arena, "value: {:.2}", &values, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "value: 42"); // Debug formatting ignores precision for integers
}

#[test] 
#[should_panic(expected = "precision with debug (?), use either {:.N} or {:?}")]
fn test_text_format_invalid_precision_debug() {
    let mut arena: Arena<1024> = Arena::new();

    // This should panic with a clear error message
    let values = [42];
    let _text = Text::format_dbg(&mut arena, "value: {:.1?}", &values, "");
}

#[test]
#[should_panic(expected = "Invalid format string: found '{' without matching '}'")]  
fn test_text_format_unmatched_brace_open() {
    let mut arena: Arena<1024> = Arena::new();

    let values = [42];
    let _text = Text::format_display(&mut arena, "value: {", &values, "");
}

#[test]
#[should_panic(expected = "Invalid format string: found '}' without matching '{'")]
fn test_text_format_unmatched_brace_close() {
    let mut arena: Arena<1024> = Arena::new();

    let values = [42];
    let _text = Text::format_display(&mut arena, "value: }", &values, "");
}

#[test]
#[should_panic(expected = "Invalid format specifier: supported formats are {}, {:?}, {:.N}")]
fn test_text_format_invalid_specifier() {
    let mut arena: Arena<1024> = Arena::new();

    let values = [42];
    let _text = Text::format_display(&mut arena, "value: {:x}", &values, "");
}

#[test]
fn test_text_format_with_second_message() {
    let mut arena: Arena<1024> = Arena::new();

    // Test format_display with second message
    let values = [42, 100];
    let text = Text::format_display(&mut arena, "Values: {}, {}", &values, " (end)").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "Values: 42, 100 (end)");

    // Test format_dbg with second message
    let values = ["hello"];
    let text = Text::format_dbg(&mut arena, "Debug: {}", &values, " done").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "Debug: \"hello\" done");

    // Test with empty second message (should work like before)
    let values = [3.14];
    let text = Text::format_display(&mut arena, "Pi: {:.2}", &values, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "Pi: 3.14");
}

#[test]
fn test_text_format_practical_two_message_examples() {
    let mut arena: Arena<1024> = Arena::new();

    // Example: "Temperature: 23 degrees"
    let temp_values = [23];
    let text = Text::format_display(&mut arena, "Temperature: {}", &temp_values, " degrees").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "Temperature: 23 degrees");

    // Example: "Loading... 45% complete"
    let progress_values = [45];
    let text = Text::format_display(&mut arena, "Loading... {}%", &progress_values, " complete").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "Loading... 45% complete");

    // Example with debug: "Player position: (10, 20) in world space"
    let pos_values = [(10, 20)];
    let text = Text::format_dbg(&mut arena, "Player position: {}", &pos_values, " in world space").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "Player position: (10, 20) in world space");
}